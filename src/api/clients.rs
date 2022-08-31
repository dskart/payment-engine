use crate::{
    api::{session::Context, transactions::TransactionResponse, Route},
    app::SanitizedError,
    model,
};
use rocket::fairing::AdHoc;
use rocket::serde::{json::Json, Deserialize, Serialize};

const CLIENT_ROUTE_BASE: &str = "/clients";
pub struct ClientRoute {}

impl Route for ClientRoute {
    fn stage() -> AdHoc {
        AdHoc::on_ignite("Client Routing", |rocket| async {
            rocket.mount(CLIENT_ROUTE_BASE, rocket::routes![get_client, get_all_clients])
        })
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(crate = "rocket::serde")]
#[serde(rename_all = "camelCase")]
pub struct ClientResponse {
    pub id: u16,
    pub available: f32,
    pub held: f32,
    pub total: f32,
    pub locked: bool,
}

impl From<model::Client> for ClientResponse {
    fn from(c: model::Client) -> Self {
        return Self {
            id: c.id,
            available: c.available,
            held: c.held,
            total: c.total,
            locked: c.locked,
        };
    }
}

impl From<ClientResponse> for model::CSVClient {
    fn from(c: ClientResponse) -> Self {
        return Self {
            client: c.id,
            available: c.available,
            held: c.held,
            total: c.total,
            locked: c.locked,
        };
    }
}

#[rocket::get("/<id>")]
pub async fn get_client(id: u16, context: Context) -> Result<Json<ClientResponse>, SanitizedError> {
    let sess = context.session();
    if let Some(client) = sess.get_client_by_id(id).await? {
        return Ok(Json(client.into()));
    } else {
        return Err(SanitizedError::NotFound(format!("client {id:} not found")));
    }
}

#[rocket::get("/<id>/transactions")]
pub async fn get_client_transactions(id: u16, context: Context) -> Result<Json<Vec<TransactionResponse>>, SanitizedError> {
    let sess = context.session();
    let transactions: Vec<TransactionResponse> = sess
        .get_all_client_transactions(id)
        .await?
        .into_iter()
        .map(|x| TransactionResponse::from(x))
        .collect();
    return Ok(Json(transactions.into()));
}

#[rocket::get("/")]
pub async fn get_all_clients(context: Context) -> Result<Json<Vec<ClientResponse>>, SanitizedError> {
    let sess = context.session();
    let clients: Vec<ClientResponse> = sess.get_all_clients().await?.into_iter().map(|x| ClientResponse::from(x)).collect();
    return Ok(Json(clients.into()));
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::api::{transactions, API};
    use rocket::http::{ContentType, Status};
    use rocket::serde::json;

    #[rocket::async_test]
    async fn test_get_client_empty() {
        let api = API::new_test_api().await;
        let client = api.test_rocket_client().await;
        let response = client.get(rocket::uri!("/clients", get_client(1))).dispatch().await;
        assert_eq!(response.status(), Status::NotFound);
    }

    #[rocket::async_test]
    async fn test_get_client() {
        let api = API::new_test_api().await;
        let client = api.test_rocket_client().await;
        let deposit = transactions::PostTransaction {
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

        let response = client.get(rocket::uri!("/clients", get_client(1))).dispatch().await;
        assert_eq!(response.status(), Status::Ok);

        let expected = ClientResponse {
            id: 1,
            available: 10.0,
            held: 0.0,
            total: 10.0,
            locked: false,
        };
        let resp_client = response.into_json::<ClientResponse>().await.unwrap();

        assert_eq!(expected, resp_client);
    }

    #[rocket::async_test]
    async fn test_get_all_client() {
        let api = API::new_test_api().await;
        let client = api.test_rocket_client().await;
        for i in 1..10 {
            let deposit = transactions::PostTransaction {
                tx: i,
                client: i as u16,
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
        }

        let response = client.get(rocket::uri!("/clients", get_all_clients())).dispatch().await;
        assert_eq!(response.status(), Status::Ok);

        let resp_client = response.into_json::<Vec<ClientResponse>>().await.unwrap();
        assert_eq!(9, resp_client.len());
    }
}
