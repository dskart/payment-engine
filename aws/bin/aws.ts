#!/usr/bin/env node
import "source-map-support/register";
import * as cdk from "aws-cdk-lib";
import { PaymentEngineStack } from "../lib/payment-engine-stack";

const app = new cdk.App();
new PaymentEngineStack(app, "PaymentEngineStack", {
    stackName: app.node.tryGetContext("stack-name"),
    env: {
        account: process.env.CDK_DEFAULT_ACCOUNT,
        region: process.env.CDK_DEFAULT_REGION,
    },
});
