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

# ======== DynamoDB & Kinesis 接続先 ========
DYNAMODB_ENDPOINT="${DYNAMODB_ENDPOINT:-host.docker.internal:4566}"
KINESIS_ENDPOINT="${KINESIS_ENDPOINT:-host.docker.internal:4566}"
DYNAMODB_ENDPOINT_URL_OPTION="--region $REGION"
KINESIS_ENDPOINT_URL_OPTION="--region $REGION"

if [ "$ENV_NAME" = "dev" ]; then
  DYNAMODB_ENDPOINT_URL_OPTION="--endpoint-url http://${DYNAMODB_ENDPOINT} --region $REGION"
  KINESIS_ENDPOINT_URL_OPTION="--endpoint-url http://${KINESIS_ENDPOINT} --region $REGION"
fi

# ======== テーブル名・ストリーム名 ========
PREFIX="${PREFIX:-myapp}"

JOURNAL_TABLE_NAME="${JOURNAL_TABLE_NAME:-${PREFIX}-journal}"
JOURNAL_GSI_NAME="${JOURNAL_GSI_NAME:-${PREFIX}-aid-index}"
JOURNAL_STREAM_NAME="${JOURNAL_STREAM_NAME:-${PREFIX}-journal-stream}"
JOURNAL_EVENT_ID_GSI_NAME="${JOURNAL_EVENT_ID_GSI_NAME:-${PREFIX}-event-id-index}"

SNAPSHOT_TABLE_NAME="${SNAPSHOT_TABLE_NAME:-${PREFIX}-snapshot}"
SNAPSHOT_GSI_NAME="${SNAPSHOT_GSI_NAME:-${PREFIX}-aid-index}"

OUTBOX_TABLE_NAME="${OUTBOX_TABLE_NAME:-${PREFIX}-outbox}"
OUTBOX_GSI_NAME="${OUTBOX_GSI_NAME:-${PREFIX}-status-index}"
OUTBOX_STREAM_NAME="${OUTBOX_STREAM_NAME:-${PREFIX}-outbox-stream}"

INVERTED_INDEX_TABLE_NAME="${INVERTED_INDEX_TABLE_NAME:-${PREFIX}-inverted-index}"
INVERTED_INDEX_GSI_NAME="${INVERTED_INDEX_GSI_NAME:-${PREFIX}-keyword-index}"

echo "Region            = $REGION"
echo "DynamoDB endpoint = ${DYNAMODB_ENDPOINT_URL_OPTION:-(AWS cloud)}"
echo "Kinesis endpoint  = ${KINESIS_ENDPOINT_URL_OPTION:-(AWS cloud)}"
echo "Journal table     = $JOURNAL_TABLE_NAME"
echo "Journal stream    = $JOURNAL_STREAM_NAME"
echo "Snapshot table    = $SNAPSHOT_TABLE_NAME"
echo "Outbox table      = $OUTBOX_TABLE_NAME"
echo "Outbox stream     = $OUTBOX_STREAM_NAME"

# Kinesis Streamを作成する関数
create_kinesis_stream() {
  local STREAM_NAME=$1
  local SHARD_COUNT=1

  echo "Creating Kinesis Data Stream: $STREAM_NAME" >&2
  aws kinesis create-stream $KINESIS_ENDPOINT_URL_OPTION \
    --stream-name "$STREAM_NAME" \
    --shard-count $SHARD_COUNT

  echo "Waiting for stream to become active..." >&2
  aws kinesis wait stream-exists $KINESIS_ENDPOINT_URL_OPTION \
    --stream-name "$STREAM_NAME"

  echo 'sleep 5' >&2
  sleep 5

  local STREAM_ARN=""
  for i in $(seq 1 10); do
    # jqで確実にARNだけ取得
    STREAM_ARN=$(aws kinesis describe-stream $KINESIS_ENDPOINT_URL_OPTION \
      --stream-name "$STREAM_NAME" \
      --output json 2>/dev/null | jq -r '.StreamDescription.StreamARN')
    echo "STREAM_ARN: $STREAM_ARN" >&2
    if [ "$STREAM_ARN" != "null" ] && [ -n "$STREAM_ARN" ]; then
      break
    fi
    echo "Waiting for StreamARN... ($i)" >&2
    sleep 1
  done

  if [ -z "$STREAM_ARN" ] || [ "$STREAM_ARN" = "null" ]; then
    echo "Failed to get StreamARN for $STREAM_NAME" >&2
    exit 1
  fi

  echo "$STREAM_ARN"
}

# DynamoDBテーブルを作成する関数（ストリームなし）
create_snapshot_table() {
  local TABLE_NAME=$1
  local GSI_NAME=$2

  echo "Creating DynamoDB table: $TABLE_NAME"
  aws dynamodb create-table $DYNAMODB_ENDPOINT_URL_OPTION \
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
    ]"
}

create_journal_table() {
  local TABLE_NAME=$1
  local GSI_NAME=$2
  local EVENT_ID_GSI_NAME=$3

  echo "Creating DynamoDB table: $TABLE_NAME"
  aws dynamodb create-table $DYNAMODB_ENDPOINT_URL_OPTION \
    --table-name "$TABLE_NAME" \
    --attribute-definitions \
      AttributeName=pkey,AttributeType=S \
      AttributeName=skey,AttributeType=S \
      AttributeName=aid,AttributeType=S \
      AttributeName=seq_nr,AttributeType=N \
      AttributeName=event_id,AttributeType=S \
    --key-schema \
      AttributeName=pkey,KeyType=HASH \
      AttributeName=skey,KeyType=RANGE \
    --provisioned-throughput ReadCapacityUnits=10,WriteCapacityUnits=10 \
    --global-secondary-indexes "[
      {
        \"IndexName\": \"${GSI_NAME}\",
        \"KeySchema\": [
          {\"AttributeName\":\"aid\",\"KeyType\":\"HASH\"},
          {\"AttributeName\":\"seq_nr\",\"KeyType\":\"RANGE\"}
        ],
        \"Projection\": {\"ProjectionType\":\"ALL\"},
        \"ProvisionedThroughput\": {\"ReadCapacityUnits\":10,\"WriteCapacityUnits\":10}
      },
      {
        \"IndexName\": \"${EVENT_ID_GSI_NAME}\",
        \"KeySchema\": [
          {\"AttributeName\":\"event_id\",\"KeyType\":\"HASH\"}
        ],
        \"Projection\": {\"ProjectionType\":\"KEYS_ONLY\"},
        \"ProvisionedThroughput\": {\"ReadCapacityUnits\":10,\"WriteCapacityUnits\":10}
      }
    ]"
}

create_outbox_table() {
  local TABLE_NAME=$1

  echo "Creating DynamoDB table: $TABLE_NAME"
  aws dynamodb create-table $DYNAMODB_ENDPOINT_URL_OPTION \
    --table-name "$TABLE_NAME" \
    --attribute-definitions \
      AttributeName=pkey,AttributeType=S \
      AttributeName=skey,AttributeType=S \
      AttributeName=status,AttributeType=S \
    --key-schema \
      AttributeName=pkey,KeyType=HASH \
      AttributeName=skey,KeyType=RANGE \
    --provisioned-throughput ReadCapacityUnits=10,WriteCapacityUnits=10 \
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

  echo "Creating DynamoDB table: $TABLE_NAME"
  aws dynamodb create-table $DYNAMODB_ENDPOINT_URL_OPTION \
    --table-name "$TABLE_NAME" \
    --attribute-definitions \
      AttributeName=pkey,AttributeType=S \
      AttributeName=skey,AttributeType=S \
    --key-schema \
      AttributeName=pkey,KeyType=HASH \
      AttributeName=skey,KeyType=RANGE \
    --provisioned-throughput ReadCapacityUnits=10,WriteCapacityUnits=10
}

# DynamoDBテーブルとKinesis Streamを接続する関数
connect_table_to_kinesis() {
  local TABLE_NAME=$1
  local STREAM_ARN=$2

  echo "Connecting table $TABLE_NAME to Kinesis Stream $STREAM_ARN"

  # 追加の待機時間を設定
  sleep 5  # 5秒待機

  # テーブルとKinesisストリームを接続
  aws dynamodb enable-kinesis-streaming-destination $DYNAMODB_ENDPOINT_URL_OPTION \
    --table-name "$TABLE_NAME" \
    --stream-arn "$STREAM_ARN" \
    --enable-kinesis-streaming-configuration ApproximateCreationDateTimePrecision=MICROSECOND

  echo "Connected $TABLE_NAME to Kinesis Stream successfully!"
}


# ---- 実行 ----
# 1. Kinesis Streamsの作成
JOURNAL_STREAM_ARN=$(create_kinesis_stream "$JOURNAL_STREAM_NAME")
echo "Journal stream ARN: $JOURNAL_STREAM_ARN"

OUTBOX_STREAM_ARN=$(create_kinesis_stream "$OUTBOX_STREAM_NAME")
echo "Outbox stream ARN: $OUTBOX_STREAM_ARN"

# 2. DynamoDBテーブルの作成（ストリームなし）
create_journal_table "$JOURNAL_TABLE_NAME" "$JOURNAL_GSI_NAME" "$JOURNAL_EVENT_ID_GSI_NAME"
create_snapshot_table "$SNAPSHOT_TABLE_NAME" "$SNAPSHOT_GSI_NAME"
create_outbox_table "$OUTBOX_TABLE_NAME"
create_inverted_index_table "$INVERTED_INDEX_TABLE_NAME" "$INVERTED_INDEX_GSI_NAME"

# 3. DynamoDBテーブルとKinesis Streamsの接続
connect_table_to_kinesis "$JOURNAL_TABLE_NAME" "$JOURNAL_STREAM_ARN"
connect_table_to_kinesis "$OUTBOX_TABLE_NAME" "$OUTBOX_STREAM_ARN"

echo "✅ Tables and Kinesis Streams created and connected successfully in $REGION"
