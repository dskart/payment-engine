use crate::{
    api::{session::Context, Route},
    app::SanitizedError,
    model,
};
use chrono::Utc;
use rocket::fairing::AdHoc;
use rocket::serde::{json::Json, Deserialize, Serialize};

pub const TRANSACTION_ROUTE_BASE: &str = "/transactions";
pub struct TransactionRoute {}

impl Route for TransactionRoute {
    fn stage() -> AdHoc {
        AdHoc::on_ignite("Transaction Routing", |rocket| async {
            rocket.mount(TRANSACTION_ROUTE_BASE, rocket::routes![get_transaction, post_transaction])
        })
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(crate = "rocket::serde")]
#[serde(rename_all = "camelCase")]
pub struct TransactionResponse {
    #[serde(rename = "type")]
    pub record_type: model::RecordType,
    pub client: u16,
    pub tx: u32,
    pub amount: f32,
}

impl From<model::Transaction> for TransactionResponse {
    fn from(tx: model::Transaction) -> Self {
        return Self {
            record_type: tx.record_type,
            client: tx.client,
            tx: tx.tx,
            amount: tx.amount,
        };
    }
}

#[rocket::get("/<id>", format = "json")]
pub async fn get_transaction(id: u32, context: Context) -> Result<Json<TransactionResponse>, SanitizedError> {
    let sess = context.session();
    if let Some(tx) = sess.get_transaction_by_id(id).await? {
        return Ok(Json(tx.into()));
    } else {
        return Err(SanitizedError::NotFound(format!("Trasaction {id:} not found.")));
    }
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
#[serde(rename_all = "camelCase")]
pub struct PostTransaction {
    #[serde(rename = "type")]
    pub record_type: model::RecordType,
    pub client: u16,
    pub tx: u32,
    pub amount: f32,
}

impl From<PostTransaction> for model::Transaction {
    fn from(tx: PostTransaction) -> Self {
        let now = Utc::now();
        return model::Transaction {
            record_type: tx.record_type,
            client: tx.client,
            tx: tx.tx,
            amount: tx.amount,

            creation_time: now,
            revision_number: 1,
            revision_time: now,
        };
    }
}

impl From<model::Transaction> for PostTransaction {
    fn from(tx: model::Transaction) -> Self {
        return Self {
            record_type: tx.record_type,
            client: tx.client,
            tx: tx.tx,
            amount: tx.amount,
        };
    }
}

#[rocket::post("/", format = "json", data = "<tx_json>")]
pub async fn post_transaction(tx_json: Json<PostTransaction>, context: Context) -> Result<(), SanitizedError> {
    let sess = context.session();
    let tx: PostTransaction = tx_json.into_inner();
    return sess.process_transaction(tx.into()).await;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::api::{transactions, API};
    use rocket::http::{ContentType, Status};
    use rocket::serde::json;

    #[rocket::async_test]
    async fn test_get_transaction_empty() {
        let api = API::new_test_api().await;
        let client = api.test_rocket_client().await;
        let response = client.get(rocket::uri!("/transactions", get_transaction(1))).dispatch().await;
        assert_eq!(response.status(), Status::NotFound);
    }

    #[rocket::async_test]
    async fn test_get_transaction() {
        let api = API::new_test_api().await;
        let client = api.test_rocket_client().await;
        let deposit = PostTransaction {
            tx: 1,
            client: 1,
            record_type: model::RecordType::Deposit,
            amount: 10.0,
        };
        let response = client
            .post(rocket::uri!("/transactions", transactions::post_transaction()))
            .header(ContentType::JSON)
            .body(json::to_string(&deposit).unwrap())
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);

        let response = client.get(rocket::uri!("/transactions", get_transaction(1))).dispatch().await;
        assert_eq!(response.status(), Status::Ok);

        let expected = TransactionResponse {
            tx: 1,
            client: 1,
            record_type: model::RecordType::Deposit,
            amount: 10.0,
        };
        let resp_tx = response.into_json::<TransactionResponse>().await.unwrap();

        assert_eq!(expected, resp_tx);
    }
}
