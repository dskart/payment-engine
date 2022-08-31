use crate::{
    app::{distant_future, distant_past, SanitizedResult, Session},
    model::client::*,
};
use chrono::{DateTime, Utc};

impl Session<'_> {
    pub async fn get_clients_by_time_range(&self, min_time: DateTime<Utc>, max_time: DateTime<Utc>, limit: i32) -> SanitizedResult<Vec<Client>> {
        Ok(self.sanitize(self.store.get_clients_by_time_range(min_time, max_time, limit).await)?)
    }

    pub async fn get_client_by_id(&self, id: u16) -> SanitizedResult<Option<Client>> {
        Ok(self.sanitize(self.store.get_client_by_id(id).await)?)
    }

    pub async fn get_all_clients(&self) -> SanitizedResult<Vec<Client>> {
        Ok(self.get_clients_by_time_range(distant_past(), distant_future(), 0).await?)
    }

    pub async fn add_client(&self, client: &Client) -> SanitizedResult<()> {
        Ok(self.sanitize(self.store.add_client(client).await)?)
    }
}
