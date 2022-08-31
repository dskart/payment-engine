# Payment Engine :bank:

This is a simple payment engine toy. It takes in a series of transactions (more detail about transactions here: [transactions.md](./transactions.md)) and popluate client accounts depending on those transactions.

Clients can be repesented like so:

client | available | held | total | locked
------ | --------- | ---- | ----- | ------
1 | 1.5 | 0.0 | 1.5 | false
2 | 2.0 | 0.0 | 2.0 | false

## Getting Started

Make sure to have rust installed on your computer to run this app

## Running As A Simple Enginer

You can simply run this engine by passing in a `.cvs` file full of transactions like so:

```bash
cargo run -- transactions.csv > accounts.csv
```

The resulting client accounts will be outputed to the terminal in a csv format

## Running As A Service

You can also run this engine as a full fledge service with a database behind a REST api built with [Rocket](https://rocket.rs/).

### Prerequisites

You might want to create a `config.yaml` to configure the type of store you want to use. You can select which kind of store in the yaml file like so:

```yaml
App:
  Store:
    RedisAddress: 127.0.0.1:6379
    # InMemory: True 
```

If using redis as your local backend, you might want to start a local redis instance:

```bash
redis-serve
```

### Running

You can then run the service locally with the following commands:

``` bash
cargo run -- serve --help
cargo run -- serve
```

This will serve the service on [http://localhost:8080/](http://localhost:8080/)

### Enpoints

#### /clients

- `GET clients/<id>` returns a specific client account
- `GET clients/<id>/transactions` returns all tx for a client
- `GET clients/` returns list of all clients accounts

#### /transactions

- `GET transactions/<id>` returns a transaction
- `POST transactions/` processes a transaction

#### /healthz

- `GET /healthz` healtz

### Stores

You can configure 3 types of stores backend for your engine:

- In Memory (great for unit/integrationtesting)
- Redis (great for local dev)
- DynamoDB (great for cloud deployment)

The different configs can be found in [src/store/config.rs](./src/store/config.rs) and defined in your config.yaml

### Docker Compose

You can simply run the service in docker-compose with a redis backend using the following commands:

```bash
docker-compose up
docker-compose down
```

## Testing

Most of the testing is done through integration tests in the [./tests](./tests/) folder.

The integration tests work by taking in a transactions csv file and "expected" csv file containing all of the final clients accounts. It then run the engine on the transactions csv file and assert if the final clients account match what is expected.

### Generating test data

You can generate test data by editing and running the [./test_data_generator](./test_data_generator.py) file like so:

```bash
pipenv run python ./test_data_generator.py
```

### Benchmarks

There is a benchmark integration test that runs a test data csv pair throught the payment engine REST api.

You can run benchmarks like so:

```bash
cargo +nightly bench --test '*' --features benchmarks
```

## Deploying

This service is deployable as a cloudformation stack on aws using cdk.

The stack deploys the service as a docker container in a ecs fargate cluster in 3 az zones behind a Network Load Balancer (NLB). It also uses a DyanmoDB table as the store backend instead of Redis or InMemory.

See [aws/README.md](./aws/README.md) for more details how to deploy this stack in your account.
