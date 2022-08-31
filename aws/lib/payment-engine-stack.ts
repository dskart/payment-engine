import {
    Stack,
    StackProps,
    aws_ec2 as ec2,
    Tags,
    Aws,
    aws_iam as iam,
    aws_logs as logs,
    aws_ecs as ecs,
    aws_ecr as ecr,
    aws_ecs_patterns as ecsPatterns,
} from "aws-cdk-lib";
import { Construct } from "constructs";
import { createDefaultDynamodbTable } from "./default-dynamo-table";
import { CfnParameters } from "./cfn-parameters";

export class PaymentEngineStack extends Stack {
    constructor(scope: Construct, id: string, props?: StackProps) {
        super(scope, id, props);

        const params = new CfnParameters(this);
        const vpc = new ec2.Vpc(this, "VPC", {
            vpcName: "VPC",
            maxAzs: 3,
        });
        Tags.of(vpc).add("Name", Aws.STACK_NAME);

        // ---------------------- ECR ------------------------
        // Docker image, log group and DynamoDB table
        const logGroup = new logs.LogGroup(this, "LogGrou");
        const table = createDefaultDynamodbTable(this, "Table");

        const image = ecs.ContainerImage.fromEcrRepository(
            ecr.Repository.fromRepositoryName(
                this,
                "Image",
                params.ecrRepositoryName.valueAsString
            ),
            params.imageTag.valueAsString
        );

        // ----------------- ECS Task ----------------------
        // Create a role that can be assumed by ecs tasks
        const taskRole = new iam.Role(this, "TaskRole", {
            assumedBy: new iam.ServicePrincipal("ecs-tasks.amazonaws.com"),
        });

        // Grant "full access" to our ecs task role, this will allow any ecs task
        // with this role to access the dynamoDB table
        table.grantFullAccess(taskRole);

        // Create the task definition and give it the right networkMode and role
        const taskDefinition = new ecs.FargateTaskDefinition(
            this,
            "TaskDefinition",
            {
                cpu: 512,
                memoryLimitMiB: 1024,
                taskRole,
            }
        );

        // Add our service container to the task with commands, env variables, logging and other configurations
        taskDefinition.addContainer("payment-engine", {
            image: image,
            containerName: "payment-engine",
            command: ["serve", "-p", "80"],
            environment: {
                // eslint-disable-next-line @typescript-eslint/naming-convention
                CDK_APP_STORE_DYNAMODB_TABLENAME: table.tableName,
                // eslint-disable-next-line @typescript-eslint/naming-convention
                AWS_REGION: Aws.REGION,
            },
            memoryReservationMiB: 500,
            portMappings: [{ containerPort: 80 }],
            logging: ecs.LogDriver.awsLogs({
                logGroup: logGroup,
                streamPrefix: Aws.STACK_NAME,
            }),
        });

        // ----------------- ECS Cluster ----------------------
        const cluster = new ecs.Cluster(this, "ECSCluster", { vpc: vpc });

        // ----------------- ECS Service ----------------------
        // This is a nice construct that wraps everything nicely for us.
        const ecsService = new ecsPatterns.NetworkLoadBalancedFargateService(
            this,
            "ECSService",
            {
                cluster: cluster,
                desiredCount: 3,
                publicLoadBalancer: true,
                taskDefinition: taskDefinition,
            }
        );
        ecsService.service.connections.allowFrom(
            ec2.Peer.ipv4(vpc.vpcCidrBlock),
            ec2.Port.tcp(80)
        );
        ecsService.targetGroup.setAttribute(
            "deregistration_delay.timeout_seconds",
            "60"
        );
    }
}
