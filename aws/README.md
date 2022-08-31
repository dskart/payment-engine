# Welcome to your CDK TypeScript project

This is a blank project for CDK development with TypeScript.

The `cdk.json` file tells the CDK Toolkit how to execute your app.

## Useful commands

* `npm run build`   compile typescript to js
* `npm run watch`   watch for changes and compile
* `npm run test`    perform the jest unit tests
* `cdk deploy`      deploy this stack to your default AWS account/region
* `cdk diff`        compare deployed stack with current state
* `cdk synth`       emits the synthesized CloudFormation template

## Deploying To A New Account

### Prerequisites

* Make sure to create the required ECR repos and populate them with the corresponding images. To do this, either pull the images from a dev/prod account or build your own image locally with docker and upload it to the new account ecr. You need to create `payment-engine` ecr repository.

### Deploying

You can then use the cdk with the following command:

```bash
cdk synth
cdk deploy 
```

You might want to change the following parameters depending on your account and needs:

* EcrRepositoryName
* ImageTag

```bash
cdk deploy --parameters EcrRepositoryName=YOUR_VALUE --parameters ImageTag=YOUR_VALUE
```
