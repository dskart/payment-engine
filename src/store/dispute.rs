use crate::store_key;
use crate::{
    model,
    store::{client::*, Error, Result, Store},
};
use keyvaluestore::{AtomicWriteOperation, Backend};

pub const DISPUTE_KEY: &str = "dispute";
pub const DISPUTE_REVISION_KEY: &str = "dispute_revision";
pub const REFERENCE_TX_DISPUTE_KEY: &str = "reference_tx_dispute";
pub const CLIENT_DISPUTES_SET_KEY: &str = "client_disputes";

impl<B: Backend + Sync> Store<B> {
    pub async fn process_dispute(&self, client: &model::Client, dispute: &model::Dispute) -> Result<()> {
        let serialized = Self::serialize(&dispute)?;
        let reference_tx_id = model::Id::from(dispute.referenced_tx.tx);

        let serialized_client = Self::serialize(client)?;
        let client_id = model::Id::from(client.id as u32);

        let mut tx = AtomicWriteOperation::new();
        // update client with new available/held values
        tx.z_add(CLIENTS_SET_KEY, client_id.as_ref(), Self::time_microsecond_score(&client.revision_time));
        tx.set(store_key!(CLIENT_KEY, ":", client_id), &serialized_client);
        tx.set_nx(store_key!(CLIENT_REVISION_KEY, ":", client_id, ":", client.revision_number), &serialized_client);

        // add a dispute
        tx.set_nx(store_key!(DISPUTE_KEY, ":", dispute.id), &serialized);
        tx.set_nx(store_key!(DISPUTE_REVISION_KEY, ":", dispute.id, ":", dispute.revision_number), &serialized);
        tx.set_nx(store_key!(REFERENCE_TX_DISPUTE_KEY, ":", reference_tx_id), &serialized);
        tx.z_add(
            store_key!(CLIENT_DISPUTES_SET_KEY, ":", client_id),
            dispute.id.as_ref(),
            Self::time_microsecond_score(&dispute.revision_time),
        );

        match self.backend.exec_atomic_write(tx).await? {
            true => Ok(()),
            false => Err(Error::Contention),
        }
    }

    // This removes the dispute from the tx and updates the client accordingly
    pub async fn remove_dispute(&self, client: &model::Client, dispute: &model::Dispute) -> Result<()> {
        let serialized = Self::serialize(&dispute)?;
        let reference_tx_id = model::Id::from(dispute.referenced_tx.tx);

        let serialized_client = Self::serialize(client)?;
        let client_id = model::Id::from(client.id as u32);

        let mut tx = AtomicWriteOperation::new();
        // update client with new available/held values
        tx.z_add(CLIENTS_SET_KEY, client_id.as_ref(), Self::time_microsecond_score(&client.revision_time));
        tx.set(store_key!(CLIENT_KEY, ":", client_id), &serialized_client);
        tx.set_nx(store_key!(CLIENT_REVISION_KEY, ":", client_id, ":", client.revision_number), &serialized_client);

        // remove dispute
        tx.set(store_key!(DISPUTE_KEY, ":", dispute.id), &serialized);
        tx.set_nx(store_key!(DISPUTE_REVISION_KEY, ":", dispute.id, ":", dispute.revision_number), &serialized);
        tx.set(store_key!(REFERENCE_TX_DISPUTE_KEY, ":", reference_tx_id), &serialized);
        tx.z_rem(store_key!(CLIENT_DISPUTES_SET_KEY, ":", client_id), dispute.id.as_ref());

        match self.backend.exec_atomic_write(tx).await? {
            true => Ok(()),
            false => Err(Error::Contention),
        }
    }

    pub async fn get_dispute_by_reference_tx_id(&self, tx_id: u32) -> Result<Option<model::Dispute>> {
        let id = model::Id::from(tx_id as u32);
        if let Some(v) = self.backend.get(store_key!(REFERENCE_TX_DISPUTE_KEY, ":", id)).await? {
            let ret: model::Dispute = Self::deserialize(v.as_ref())?;
            return Ok(Some(ret));
        } else {
            return Ok(None);
        }
    }
}
