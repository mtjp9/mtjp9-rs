FROM public.ecr.aws/aws-cli/aws-cli:latest

# jqをインストール
RUN yum install -y jq

ADD *.sh ./

ENTRYPOINT ["/bin/sh", "-c", "echo 'Waiting for LocalStack to be ready...' && until curl -s http://localstack:4566/health > /dev/null 2>&1; do echo 'Waiting...'; sleep 2; done && ./create-tables.sh \"$@\"", "--"]
