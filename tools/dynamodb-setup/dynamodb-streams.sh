#!/usr/bin/env sh

set -eu
cd "$(dirname "$0")"

usage() {
  cat <<EOF
Usage: $0 [-e dev|prod] [-r region]
  -e  Environment name (dev なら LocalStack エンドポイントを自動付与)
  -r  AWS region (default: ap-northeast-1)
EOF
  exit 1
}

# ======== デフォルト値 ========
ENV_NAME=""
REGION="ap-northeast-1"

# ======== 引数パース ========
while getopts "e:r:" OPT; do
  case "$OPT" in
    e) ENV_NAME="$OPTARG" ;;
    r) REGION="$OPTARG" ;;
    *) usage ;;
  esac
done

# ======== 環境変数 ========
export AWS_DEFAULT_REGION="$REGION"
export AWS_REGION="$REGION"
export AWS_ACCESS_KEY_ID="${AWS_ACCESS_KEY_ID:-x}"
export AWS_SECRET_ACCESS_KEY="${AWS_SECRET_ACCESS_KEY:-x}"
export AWS_PAGER=""

# ======== DynamoDB 接続先 ========
DYNAMODB_ENDPOINT="${DYNAMODB_ENDPOINT:-localhost:31566}"
ENDPOINT_URL_OPTION="--region $REGION"
[ "$ENV_NAME" = "dev" ] && \
  ENDPOINT_URL_OPTION="--endpoint-url http://${DYNAMODB_ENDPOINT} --region $REGION"

# ======== テーブル名 ========
PREFIX="${PREFIX:-myapp}"

JOURNAL_TABLE_NAME="${JOURNAL_TABLE_NAME:-${PREFIX}-journal}"
JOURNAL_GSI_NAME="${JOURNAL_GSI_NAME:-${PREFIX}-aid-index}"

SNAPSHOT_TABLE_NAME="${SNAPSHOT_TABLE_NAME:-${PREFIX}-snapshot}"
SNAPSHOT_GSI_NAME="${SNAPSHOT_GSI_NAME:-${PREFIX}-aid-index}"

OUTBOX_TABLE_NAME="${OUTBOX_TABLE_NAME:-${PREFIX}-outbox}"
OUTBOX_GSI_NAME="${OUTBOX_GSI_NAME:-${PREFIX}-status-index}"

INVERTED_INDEX_TABLE_NAME="${INVERTED_INDEX_TABLE_NAME:-${PREFIX}-inverted-index}"
INVERTED_INDEX_GSI_NAME="${INVERTED_INDEX_GSI_NAME:-${PREFIX}-keyword-index}"

echo "Region            = $REGION"
echo "DynamoDB endpoint = ${ENDPOINT_URL_OPTION:-(AWS cloud)}"
echo "Journal table     = $JOURNAL_TABLE_NAME"
echo "Snapshot table    = $SNAPSHOT_TABLE_NAME"
echo "Outbox table      = $OUTBOX_TABLE_NAME"

create_common_table() {
  local TABLE_NAME=$1
  local GSI_NAME=$2
  shift 2
  local EXTRA_ARGS="$@"

  aws dynamodb create-table $ENDPOINT_URL_OPTION \
    --table-name "$TABLE_NAME" \
    --attribute-definitions \
      AttributeName=pkey,AttributeType=S \
      AttributeName=skey,AttributeType=S \
      AttributeName=aid,AttributeType=S \
      AttributeName=seq_nr,AttributeType=N \
    --key-schema \
      AttributeName=pkey,KeyType=HASH \
      AttributeName=skey,KeyType=RANGE \
    --provisioned-throughput ReadCapacityUnits=10,WriteCapacityUnits=10 \
    --stream-specification StreamEnabled=true,StreamViewType=NEW_IMAGE \
    --global-secondary-indexes "[
      {
        \"IndexName\": \"${GSI_NAME}\",
        \"KeySchema\": [
          {\"AttributeName\":\"aid\",\"KeyType\":\"HASH\"},
          {\"AttributeName\":\"seq_nr\",\"KeyType\":\"RANGE\"}
        ],
        \"Projection\": {\"ProjectionType\":\"ALL\"},
        \"ProvisionedThroughput\": {\"ReadCapacityUnits\":10,\"WriteCapacityUnits\":10}
      }
    ]" \
    $EXTRA_ARGS
}

create_outbox_table() {
  local TABLE_NAME=$1

  aws dynamodb create-table $ENDPOINT_URL_OPTION \
    --table-name "$TABLE_NAME" \
    --attribute-definitions \
      AttributeName=pkey,AttributeType=S \
      AttributeName=skey,AttributeType=S \
      AttributeName=status,AttributeType=S \
    --key-schema \
      AttributeName=pkey,KeyType=HASH \
      AttributeName=skey,KeyType=RANGE \
    --provisioned-throughput ReadCapacityUnits=10,WriteCapacityUnits=10 \
    --stream-specification StreamEnabled=true,StreamViewType=NEW_IMAGE \
    --global-secondary-indexes "[
      {
        \"IndexName\": \"${OUTBOX_GSI_NAME}\",
        \"KeySchema\": [
          {\"AttributeName\":\"status\",\"KeyType\":\"HASH\"},
          {\"AttributeName\":\"skey\",\"KeyType\":\"RANGE\"}
        ],
        \"Projection\": {\"ProjectionType\":\"ALL\"},
        \"ProvisionedThroughput\": {\"ReadCapacityUnits\":10,\"WriteCapacityUnits\":10}
      }
    ]"
}

create_inverted_index_table() {
  local TABLE_NAME=$1
  local GSI_NAME=$2

  aws dynamodb create-table $ENDPOINT_URL_OPTION \
    --table-name "$TABLE_NAME" \
    --attribute-definitions \
      AttributeName=pkey,AttributeType=S \
      AttributeName=skey,AttributeType=S \
    --key-schema \
      AttributeName=pkey,KeyType=HASH \
      AttributeName=skey,KeyType=RANGE \
    --provisioned-throughput ReadCapacityUnits=10,WriteCapacityUnits=10
}

# ---- 実行 ----
create_common_table "$JOURNAL_TABLE_NAME"  "$JOURNAL_GSI_NAME"

create_common_table "$SNAPSHOT_TABLE_NAME" "$SNAPSHOT_GSI_NAME"

create_outbox_table "$OUTBOX_TABLE_NAME" "$OUTBOX_GSI_NAME"

create_inverted_index_table "$INVERTED_INDEX_TABLE_NAME" "$INVERTED_INDEX_GSI_NAME"

echo "✅ Tables created successfully in $REGION"
