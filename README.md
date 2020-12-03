<p align="center">
  <img width="200" height="100" src="/darkowl.png">
</p>
<h1 align="center">Darkowl</h1>

An infrastructure-forward modern worker template built with Rust for AWS Lambda


## Building the lambda

```bash
cargo build --release --target x86_64-unknown-linux-musl
```

## Testing the Lambda Locally

1. Run the lambda ci docker image 

```bash
docker run --rm \
-e DOCKER_LAMBDA_STAY_OPEN=1 -p 9001:9001 \
-v "$PWD"/target/x86_64-unknown-linux-musl/release/bootstrap:/var/task/bootstrap:ro,delegated \
lambci/lambda:provided main
```

2. In a separate terminal, invoke the lambda

```bash
aws lambda invoke \
--endpoint http://localhost:9001 \
--no-sign-request --function-name=darkowl \
--invocation-type=RequestResponse \
--payload $(echo '{ "externalRequester": "cron" }' | base64 ) \
output.json
```

## Set up for Deploys

1. Ensure you have a command line role belonging to a group with IAMFullAccess, and LambdaFullAccess

2. Using that role, run:

```bash
aws iam create-role --role-name lambda-basic-execution --assume-role-policy-document '{
    "Version": "2012-10-17",
    "Statement": [
        {
        "Effect": "Allow",
        "Principal": {
            "Service": "lambda.amazonaws.com"
        },
        "Action": "sts:AssumeRole"
        }
    ]
}'
```

3. Then:

```bash
aws iam attach-role-policy \
--role-name lambda-basic-execution \
--policy-arn arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole
```

## Deploying

1. Zip the build

```bash
zip -r9 -j bootstrap.zip ./target/x86_64-unknown-linux-musl/release/bootstrap
```

2. Deploy to a new function

```bash
AWS_ACCOUNT_ID=`aws sts get-caller-identity --query Account --output text` && \
aws lambda create-function \
--function-name darkowl \
--runtime provided \
--role arn:aws:iam::$AWS_ACCOUNT_ID:role/lambda-basic-execution \
--zip-file fileb://bootstrap.zip \
--description "Simple Rust function" \
--timeout 5 \
--handler main
```

## Updating the Deployed Function

1. Rebuild

2. Rezip

3. Run:

```bash
aws lambda update-function-code --function-name darkowl --zip-file fileb://bootstrap.zip
```

## Configuring the lambda to run on a schedule

1. Add the Cloudwatch rule

```bash
aws events put-rule \
--name darkowl-schedule \
--schedule-expression 'rate(5 minutes)'
```

2. Give the rule permission to invoke your function

```bash
aws lambda add-permission \
--function-name darkowl \
--statement-id my-scheduled-event \
--action 'lambda:InvokeFunction' \
--principal events.amazonaws.com \
--source-arn {your_rule_arn}
```

3. Create a `targets.json` file for the rule

```json
[
  {
    "Id": "1", 
    "Arn": "{your_lambda_arn}"
  }
]
```

4. Add the targets to your rule

```bash
aws events put-targets --rule darkowl-schedule --targets file://targets.json
```