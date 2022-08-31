use payment_engine::{
    api::{self, ClientResponse, PostTransaction, API},
    app::{App, Config},
    model, store, Result,
};
use rocket::local::asynchronous::Client;
use rocket::serde::json;
use rocket::{
    futures::future,
    http::{ContentType, Status},
};
use slog::o;
use std::{env, path::Path};

pub async fn new_test_app<F: FnOnce(&mut Config)>(configure: F) -> App {
    let mut config = Config {
        store: store::Config {
            in_memory: true,
            ..Default::default()
        },
        ..Default::default()
    };
    configure(&mut config);
    App::new_with_config(config).await.expect("failed to create test app")
}

pub fn test_logger() -> slog::Logger {
    use slog::Drain;
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().filter_level(slog::Level::Warning).fuse();
    slog::Logger::root(drain, o!())
}

pub async fn new_test_api() -> API {
    let a = new_test_app(|_| {}).await;
    return API::new(test_logger(), a);
}

pub async fn test_rocket_client(api: API) -> Client {
    let r = api.rocket(8080);
    let client = Client::tracked(r.unwrap()).await.unwrap();
    return client;
}

pub fn parse_expected_data_csv(file_path: String) -> Result<Vec<model::CSVClient>> {
    let mut rdr = csv::ReaderBuilder::new().has_headers(true).trim(csv::Trim::All).from_path(file_path)?;
    let mut raw_record = csv::ByteRecord::new();
    let headers = rdr.byte_headers()?.clone();

    let mut ret = Vec::new();
    while rdr.read_byte_record(&mut raw_record)? {
        let client: model::CSVClient = raw_record.deserialize(Some(&headers))?;
        ret.push(client);
    }
    return Ok(ret);
}

pub async fn test_csv_data(file_name: String) {
    let app = new_test_app(|_| {}).await;
    let sess = app.new_session(test_logger());

    let test_data_path = env::current_dir().unwrap().join(Path::new("tests/test_data/"));
    let test_data_csv = test_data_path.join(file_name.clone() + ".csv").to_str().unwrap().to_string();
    let expected_data_csv = test_data_path.join(file_name + "_expected.csv").to_str().unwrap().to_string();

    sess.process_csv(test_data_csv).await.unwrap();

    let mut all_clients = sess
        .get_all_clients()
        .await
        .unwrap()
        .into_iter()
        .map(|x| model::CSVClient::from(x))
        .collect::<Vec<model::CSVClient>>();

    let mut expected_clients = parse_expected_data_csv(expected_data_csv).unwrap();
    all_clients.sort();
    expected_clients.sort();

    assert_eq!(all_clients.len(), expected_clients.len());
    assert_eq!(all_clients, expected_clients);
}

pub async fn test_service_from_csv_data(file_name: String) {
    let api = new_test_api().await;
    let rocket_client = test_rocket_client(api).await;

    let test_data_path = env::current_dir().unwrap().join(Path::new("tests/test_data/"));
    let test_data_csv = test_data_path.join(file_name.clone() + ".csv").to_str().unwrap().to_string();
    let expected_data_csv = test_data_path.join(file_name + "_expected.csv").to_str().unwrap().to_string();

    // Read test data and send to user
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(test_data_csv)
        .unwrap();

    let mut raw_record = csv::ByteRecord::new();
    let headers = rdr.byte_headers().unwrap().clone();
    while rdr.read_byte_record(&mut raw_record).unwrap() {
        let csv_transaction: model::CSVTransaction = raw_record.deserialize(Some(&headers)).unwrap();
        let transaction = PostTransaction::from(model::Transaction::from(csv_transaction));
        rocket_client
            .post(rocket::uri!("/transactions", api::post_transaction()))
            .header(ContentType::JSON)
            .body(json::to_string(&transaction).unwrap())
            .dispatch()
            .await;
    }

    // get all clients account from service and compare to expected data
    let response = rocket_client.get(rocket::uri!("/clients", api::get_all_clients())).dispatch().await;
    assert_eq!(response.status(), Status::Ok);

    let mut all_clients = response
        .into_json::<Vec<ClientResponse>>()
        .await
        .unwrap()
        .into_iter()
        .map(|x| model::CSVClient::from(x))
        .collect::<Vec<model::CSVClient>>();

    let mut expected_clients = parse_expected_data_csv(expected_data_csv).unwrap();

    all_clients.sort();
    expected_clients.sort();

    assert_eq!(all_clients.len(), expected_clients.len());
    assert_eq!(all_clients, expected_clients);
}

pub async fn test_service_from_csv_data_concurently(file_name: String) {
    let api = new_test_api().await;
    let rocket_client = test_rocket_client(api).await;

    let test_data_path = env::current_dir().unwrap().join(Path::new("tests/test_data/"));
    let test_data_csv = test_data_path.join(file_name.clone() + ".csv").to_str().unwrap().to_string();
    let expected_data_csv = test_data_path.join(file_name + "_expected.csv").to_str().unwrap().to_string();

    // Read test data and send to user
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(test_data_csv)
        .unwrap();

    let mut raw_record = csv::ByteRecord::new();
    let headers = rdr.byte_headers().unwrap().clone();
    let mut handles = Vec::new();
    while rdr.read_byte_record(&mut raw_record).unwrap() {
        let csv_transaction: model::CSVTransaction = raw_record.deserialize(Some(&headers)).unwrap();
        let transaction = PostTransaction::from(model::Transaction::from(csv_transaction));
        let req = rocket_client
            .post(rocket::uri!("/transactions", api::post_transaction()))
            .header(ContentType::JSON)
            .body(json::to_string(&transaction).unwrap())
            .dispatch();

        handles.push(req)
    }

    future::join_all(handles).await;

    // get all clients account from service and compare to expected data
    let response = rocket_client.get(rocket::uri!("/clients", api::get_all_clients())).dispatch().await;
    assert_eq!(response.status(), Status::Ok);

    let mut all_clients = response
        .into_json::<Vec<ClientResponse>>()
        .await
        .unwrap()
        .into_iter()
        .map(|x| model::CSVClient::from(x))
        .collect::<Vec<model::CSVClient>>();

    let mut expected_clients = parse_expected_data_csv(expected_data_csv).unwrap();

    all_clients.sort();
    expected_clients.sort();

    assert_eq!(all_clients.len(), expected_clients.len());
    assert_eq!(all_clients, expected_clients);
}
