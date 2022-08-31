use crate::store_key;
use crate::{
    model,
    model::client::*,
    store::{Error, Result, Store},
};
use chrono::{DateTime, Utc};
use keyvaluestore::{AtomicWriteOperation, Backend};

pub const CLIENTS_SET_KEY: &str = "clients";
pub const CLIENT_KEY: &str = "client";
pub const CLIENT_REVISION_KEY: &str = "client_revision";

impl<B: Backend + Sync> Store<B> {
    pub async fn add_client(&self, client: &Client) -> Result<()> {
        let serialized = Self::serialize(&client)?;
        let id = model::Id::from(client.id as u32);

        let mut tx = AtomicWriteOperation::new();
        tx.z_add(CLIENTS_SET_KEY, id.as_ref(), Self::time_microsecond_score(&client.revision_time));
        tx.set_nx(store_key!(CLIENT_KEY, ":", id), &serialized);
        tx.set_nx(store_key!(CLIENT_REVISION_KEY, ":", id, ":", client.revision_number), &serialized);
        match self.backend.exec_atomic_write(tx).await? {
            true => Ok(()),
            false => Err(Error::Contention),
        }
    }

    pub async fn add_client_revision(&self, client: &Client) -> Result<()> {
        let serialized = Self::serialize(&client)?;
        let id = model::Id::from(client.id as u32);

        let mut tx = AtomicWriteOperation::new();
        tx.z_add(CLIENTS_SET_KEY, id.as_ref(), Self::time_microsecond_score(&client.revision_time));
        tx.set(store_key!(CLIENT_KEY, ":", id), &serialized);
        tx.set_nx(store_key!(CLIENT_REVISION_KEY, ":", id, ":", client.revision_number), &serialized);
        match self.backend.exec_atomic_write(tx).await? {
            true => Ok(()),
            false => Err(Error::Contention),
        }
    }

    pub async fn get_client_by_id(&self, client_id: u16) -> Result<Option<Client>> {
        let id = model::Id::from(client_id as u32);
        if let Some(v) = self.backend.get(store_key!(CLIENT_KEY, ":", id)).await? {
            let ret: Client = Self::deserialize(v.as_ref())?;
            return Ok(Some(ret));
        } else {
            return Ok(None);
        }
    }

    // Gets clients within an inclusive time range. If limit is non-zero, the returned events will
    // be limited to that number. If limit is negative, the returned clients will be the last clients
    // in the range.
    pub async fn get_clients_by_time_range(&self, min: DateTime<Utc>, max: DateTime<Utc>, limit: i32) -> Result<Vec<Client>> {
        self.get_by_time_range(CLIENTS_SET_KEY, min, max, limit, CLIENT_KEY).await
    }
}
