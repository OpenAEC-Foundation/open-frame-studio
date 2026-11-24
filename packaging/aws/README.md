# Headless estimate API

`ofs-cloud --service` runs the Rust core’s fixed-frame calculation as a stateless HTTP service. It uses the core’s default rectangular frame and price table. Monetary fields are indicative amounts in EUR. The interactive desktop and the graphical SuperCloud container have separate entry points.

## HTTP contract

| Request | Result |
| --- | --- |
| `GET /api/v1/health` | Public to callers on the service network. Returns `{"status":"ok","version":"…"}`. |
| `POST /api/v1/estimates/fixed-frame` | Requires `Authorization: Bearer <token>` and `Content-Type: application/json`. Send `{"widthMm":1200,"heightMm":1400}`. |

Both dimensions are numbers in millimetres. Width ranges from 200 to 6000 and height from 200 to 4000, inclusive. A success response includes `totalCost`, `materialCost`, `glassCost`, `hardwareCost`, `gasketCost`, `panelCost`, `surfaceTreatmentCost`, `laborHours`, `laborCost`, `nlSfb` and `lineItems`. Each line contains a description, classification, quantity, unit, unit price and total. The service does not persist requests or expose the experimental project storage routes.

| Status | Condition |
| --- | --- |
| 400 | Malformed JSON |
| 401 | Missing or incorrect bearer credential; includes `WWW-Authenticate: Bearer` |
| 404 | Unknown route |
| 413 | Body exceeds 4096 bytes |
| 415 | Unsupported content type |
| 422 | Missing, unknown or mistyped field, or dimension outside the accepted range |

For a local check, use two terminals. Generate a new token in the first, make it available privately to the second and run the smoke script there. The token must contain at least 32 printable, non-space ASCII characters. `OFS_LISTEN_ADDR` defaults to `127.0.0.1:3456` for a local process.

```sh
export OFS_API_TOKEN="$(openssl rand -hex 32)"
cargo run --locked -p ofs-cloud -- --service
```

```sh
node packaging/aws/smoke.mjs http://127.0.0.1:3456
```

## Deploy to private ECS Fargate

The stack runs one Linux x86-64 task with 0.25 vCPU and 512 MiB, a read-only root filesystem and a non-root process. The internal HTTPS load balancer accepts requests from the supplied client security group. Fargate health checks and the load balancer probe `/api/v1/health`; startup output goes to CloudWatch Logs for 14 days. The execution role can pull the image, read the specified token and write to the stack log group. The application itself has no task role. [Fargate networking](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/fargate-task-networking.html) · [ECS health checks](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/healthcheck.html)

Provide an ECS cluster, a VPC with two private subnets in distinct availability zones, a client security group, a private Route 53 zone and an ACM certificate valid for the API hostname. The VPC needs private DNS and access to ECR, its S3 layer storage, Secrets Manager and CloudWatch Logs through VPC endpoints or NAT. The API security group permits outbound HTTPS. Use an ECR repository in the deployment account and region, and a Secrets Manager secret containing the raw token string encrypted with the default Secrets Manager key. After rotating the token, force a new ECS deployment so running tasks receive the new value. [Secrets Manager injection and rotation](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/secrets-envvar-secrets-manager.html)

Use an authenticated AWS CLI session in the target region. Set `REGISTRY` to the ECR registry hostname, `REPO` to the repository name and `TAG` to a unique revision such as a commit SHA. From the repository root:

```sh
aws ecr get-login-password | docker login --username AWS --password-stdin "$REGISTRY"
docker buildx build --platform linux/amd64 -f packaging/aws/Dockerfile \
  -t "$REGISTRY/$REPO:$TAG" --push .
DIGEST=$(aws ecr describe-images --repository-name "$REPO" \
  --image-ids imageTag="$TAG" --query 'imageDetails[0].imageDigest' --output text)
aws cloudformation deploy --stack-name ofs-estimate-api \
  --template-file packaging/aws/fargate.yml --capabilities CAPABILITY_IAM \
  --parameter-overrides Cluster="$CLUSTER" Vpc="$VPC_ID" PrivateSubnets="$SUBNET_A,$SUBNET_B" \
    ClientSecurityGroup="$CLIENT_SG" HostedZoneId="$ZONE_ID" ApiHostname="$API_HOST" \
    CertificateArn="$CERT_ARN" TokenSecretArn="$TOKEN_ARN" EcrRepository="$REPO" ImageDigest="$DIGEST"
```

From a client in the permitted security group that trusts the certificate, run the same HTTP checks. Record the stack ID, task revision, target health, image digest and test output for the issue. Never attach credentials. The stack creates billable ECS, load balancer and logging resources.

```sh
export OFS_API_TOKEN="$(aws secretsmanager get-secret-value --secret-id "$TOKEN_ARN" --query SecretString --output text)"
node packaging/aws/smoke.mjs "https://$API_HOST"
unset OFS_API_TOKEN
```

## Verification

On 24 November 2026, the local Windows x64 process and the Linux amd64 container passed the same HTTP checks for health, valid and invalid credentials, invalid dimensions, malformed JSON, unsupported content type, unknown fields, the request size limit and exclusion of the development project route. The two fixed-frame estimates were €330.20 for 1200 × 1400 mm and €498.62 for 2400 × 1400 mm. The script checked both totals against their line items. CI validated the CloudFormation schema and ran the container as a non-root user with a read-only filesystem. [Headless API build and test log](https://github.com/OpenAEC-Foundation/open-frame-studio/actions/runs/45405508273/job/138485028822)

A live deployment and caller test in the intended AWS VPC are pending.
