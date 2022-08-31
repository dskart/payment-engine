import { aws_dynamodb as dynamodb } from 'aws-cdk-lib';
import { Construct } from 'constructs';

export function createDefaultDynamodbTable(scope: Construct, id: string): dynamodb.Table {
    const table = new dynamodb.Table(scope, id, {
        partitionKey: { name: 'hk', type: dynamodb.AttributeType.BINARY },
        sortKey: { name: 'rk', type: dynamodb.AttributeType.BINARY },
        billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
        pointInTimeRecovery: true,
    });

    table.addLocalSecondaryIndex({
        indexName: 'rk2',
        sortKey: { name: 'rk2', type: dynamodb.AttributeType.BINARY },
    });

    return table;
}
