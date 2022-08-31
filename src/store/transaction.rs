use crate::store_key;
use crate::{
    model,
    model::client::*,
    model::transaction::*,
    store::{client::*, Error, Result, Store},
};
use chrono::{DateTime, Utc};
use keyvaluestore::{AtomicWriteOperation, Backend};

pub const TRANSACTIONS_SET_KEY: &str = "transactions";
pub const TRANSACTION_KEY: &str = "transaction";
pub const TRANSACTION_REVISION_KEY: &str = "transaction_revision";

impl<B: Backend + Sync> Store<B> {
    pub async fn process_transaction(&self, client: &Client, transaction: &Transaction) -> Result<()> {
        let serialized_client = Self::serialize(client)?;
        let client_id = model::Id::from(client.id as u32);

        let serialized_transaction = Self::serialize(transaction)?;
        let tx_id = model::Id::from(transaction.tx);

        let mut tx = AtomicWriteOperation::new();
        // update the client with new values from transaction
        tx.z_add(CLIENTS_SET_KEY, client_id.as_ref(), Self::time_microsecond_score(&client.revision_time));
        tx.set(store_key!(CLIENT_KEY, ":", client_id), &serialized_client);
        tx.set_nx(store_key!(CLIENT_REVISION_KEY, ":", client_id, ":", client.revision_number), &serialized_client);

        // add transaction into a set, client set and individually
        tx.z_add(TRANSACTIONS_SET_KEY, tx_id.as_ref(), Self::time_microsecond_score(&Utc::now()));
        tx.set_nx(store_key!(TRANSACTION_KEY, ":", tx_id), &serialized_transaction);
        tx.set_nx(
            store_key!(TRANSACTION_REVISION_KEY, ":", tx_id, ":", transaction.revision_number),
            &serialized_transaction,
        );
        tx.z_add(
            store_key!(TRANSACTIONS_SET_KEY, ":", CLIENT_KEY, ":", client_id),
            tx_id.as_ref(),
            Self::time_microsecond_score(&Utc::now()),
        );

        match self.backend.exec_atomic_write(tx).await? {
            true => Ok(()),
            false => Err(Error::Contention),
        }
    }

    pub async fn get_transaction_by_id(&self, tx_id: u32) -> Result<Option<Transaction>> {
        let id = model::Id::from(tx_id);
        if let Some(v) = self.backend.get(store_key!(TRANSACTION_KEY, ":", id)).await? {
            let ret: Transaction = Self::deserialize(v.as_ref())?;
            return Ok(Some(ret));
        } else {
            return Ok(None);
        }
    }

    pub async fn get_client_transactions_by_time_range(&self, client_id: u32, min: DateTime<Utc>, max: DateTime<Utc>, limit: i32) -> Result<Vec<Transaction>> {
        let id = model::Id::from(client_id);
        self.get_by_time_range(store_key!(TRANSACTIONS_SET_KEY, ":", CLIENT_KEY, ":", id), min, max, limit, TRANSACTION_KEY)
            .await
    }
}
