use crate::{
    app::{SanitizedResult, Session},
    model::Dispute,
};

impl Session<'_> {
    pub async fn get_dispute_by_reference_tx_id(&self, id: u32) -> SanitizedResult<Option<Dispute>> {
        Ok(self.sanitize(self.store.get_dispute_by_reference_tx_id(id).await)?)
    }
}
