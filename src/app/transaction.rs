use crate::{
    app::{distant_future, distant_past, SanitizedResult, Session},
    model::transaction::Transaction,
};

impl Session<'_> {
    pub async fn get_transaction_by_id(&self, id: u32) -> SanitizedResult<Option<Transaction>> {
        Ok(self.sanitize(self.store.get_transaction_by_id(id).await)?)
    }

    pub async fn get_all_client_transactions(&self, client_id: u16) -> SanitizedResult<Vec<Transaction>> {
        Ok(self.sanitize(
            self.store
                .get_client_transactions_by_time_range(client_id as u32, distant_future(), distant_past(), 0)
                .await,
        )?)
    }
}
