import { CfnParameter } from "aws-cdk-lib";
import { Construct } from "constructs";

export class CfnParameters {
    readonly ecrRepositoryName: CfnParameter;
    readonly imageTag: CfnParameter;
    readonly instanceCount: CfnParameter;
    readonly instanceType: CfnParameter;

    constructor(scope: Construct) {
        this.ecrRepositoryName = new CfnParameter(scope, "EcrRepositoryName", {
            type: "String",
            default: "payment-engine",
            description: "The ecr repository name",
        });
        this.ecrRepositoryName.overrideLogicalId("EcrRepositoryName");

        this.imageTag = new CfnParameter(scope, "ImageTag", {
            type: "String",
            default: "latest",
            description: "The docker image tag",
        });
        this.imageTag.overrideLogicalId("ImageTag");
    }
}
