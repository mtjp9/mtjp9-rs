#!/bin/bash

# Simple S3 bucket creation script for LocalStack
# Usage: ./create-s3.sh <bucket-name>

set -e

# Color definitions for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# LocalStack configuration
ENDPOINT_URL="${LOCALSTACK_ENDPOINT:-http://localstack:4566}"
PROFILE="${AWS_PROFILE:-localstack}"
REGION="${AWS_REGION:-ap-northeast-1}"

# Check arguments
if [ $# -lt 1 ]; then
    echo -e "${RED}Error: Please specify bucket name${NC}"
    echo "Usage: $0 <bucket-name>"
    echo "Example: $0 test-bucket"
    exit 1
fi

BUCKET_NAME=$1

# Check if AWS CLI is available
if ! command -v aws &> /dev/null; then
    echo -e "${RED}Error: AWS CLI is not installed${NC}"
    echo "Please install AWS CLI: https://aws.amazon.com/cli/"
    exit 1
fi

# Check if LocalStack is running
if ! curl -s "${ENDPOINT_URL}/_localstack/health" &> /dev/null; then
    echo -e "${YELLOW}Warning: LocalStack might not be running at ${ENDPOINT_URL}${NC}"
    echo "Make sure LocalStack is running with: docker-compose up -d"
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Operation cancelled"
        exit 1
    fi
fi

echo -e "${YELLOW}Creating S3 bucket on LocalStack...${NC}"
echo "Bucket name: $BUCKET_NAME"
echo "Endpoint: $ENDPOINT_URL"
echo "Profile: $PROFILE"
echo ""

# Create bucket
echo "Creating bucket..."
aws --endpoint-url="$ENDPOINT_URL" \
    s3api create-bucket --bucket "$BUCKET_NAME" \
    --region "$REGION" \
    --create-bucket-configuration LocationConstraint="$REGION"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Bucket '$BUCKET_NAME' created successfully${NC}"
else
    echo -e "${RED}✗ Failed to create bucket${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}=== Verifying Bucket ===${NC}"

# Check if bucket exists
echo "Checking bucket existence..."
if aws --endpoint-url="$ENDPOINT_URL" \
    s3api head-bucket --bucket "$BUCKET_NAME" 2>/dev/null; then
    echo -e "${GREEN}✓ Bucket '$BUCKET_NAME' exists${NC}"
else
    echo -e "${RED}✗ Bucket '$BUCKET_NAME' not found${NC}"
    exit 1
fi

# List all buckets
echo ""
echo -e "${GREEN}=== All Buckets in LocalStack ===${NC}"
aws --endpoint-url="$ENDPOINT_URL" s3 ls

echo ""
echo -e "${GREEN}Bucket creation completed!${NC}"
echo ""
echo "Commands to use with your bucket:"
echo "  List contents:"
echo "    aws --endpoint-url=$ENDPOINT_URL s3 ls s3://$BUCKET_NAME/"
echo ""
echo "  Upload a file:"
echo "    aws --endpoint-url=$ENDPOINT_URL s3 cp <file> s3://$BUCKET_NAME/"
echo ""
echo "  Delete the bucket:"
echo "    aws --endpoint-url=$ENDPOINT_URL s3 rb s3://$BUCKET_NAME --force"