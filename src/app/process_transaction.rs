use crate::{
    app::{SanitizedError, SanitizedResult, Session},
    model::{self, CSVClient},
    Result,
};
use std::io;

impl Session<'_> {
    pub async fn process_csv(&self, file_path: String) -> Result<()> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .trim(csv::Trim::All)
            .flexible(true)
            .from_path(file_path)?;

        let mut raw_record = csv::ByteRecord::new();
        let headers = rdr.byte_headers()?.clone();

        while rdr.read_byte_record(&mut raw_record)? {
            let csv_transaction: model::CSVTransaction = raw_record.deserialize(Some(&headers))?;
            let transaction = model::Transaction::from(csv_transaction);
            match self.process_transaction(transaction).await {
                Err(SanitizedError::UserError(msg)) => {
                    info!(self.logger(), "{msg:}"; "client_id" => transaction.client, "tx_id" => transaction.tx);
                }
                Err(err) => return Err(Box::new(err)),
                Ok(()) => {}
            };
        }

        return Ok(());
    }

    pub async fn output_all_accounts(&self) -> Result<()> {
        let all_clients = self.get_all_clients().await?;
        let mut wtr = csv::WriterBuilder::new().from_writer(io::stdout());
        for client in all_clients {
            wtr.serialize(CSVClient::from(client))?;
        }
        wtr.flush()?;

        return Ok(());
    }

    pub async fn process_transaction(&self, transaction: model::Transaction) -> SanitizedResult<()> {
        let client_id = transaction.client;
        debug!(self.logger(), "{:?}", transaction; "client_id" => transaction.client, "tx_id" => transaction.tx);

        for _ in 0..3i32 {
            let client = match self.get_client_by_id(client_id as _).await? {
                Some(c) if c.locked => return Err(SanitizedError::UserError("client is locked".to_string())),
                Some(c) => c,
                None => {
                    let new_client = model::Client::new(client_id, None);
                    match self.add_client(&new_client).await {
                        Ok(_) => new_client,
                        Err(SanitizedError::Contention) => continue,
                        Err(err) => return Err(err),
                    }
                }
            };

            let res = match transaction.record_type {
                model::RecordType::Deposit => self.deposit(client, transaction).await,
                model::RecordType::Withdrawal => self.withdrawal(client, transaction).await,
                model::RecordType::Dispute => self.dispute(client, transaction).await,
                model::RecordType::Resolve => self.resolve(client, transaction).await,
                model::RecordType::Chargeback => self.chargeback(client, transaction).await,
            };

            match res {
                Ok(_) => return Ok(()),
                Err(SanitizedError::Contention) => continue,
                Err(SanitizedError::UserError(_)) => return res,
                Err(err) => return Err(err),
            }
        }
        return Err(SanitizedError::UserError("transaction contention".to_string()));
    }

    pub async fn deposit(&self, client: model::Client, transaction: model::Transaction) -> SanitizedResult<()> {
        let available = client.available + transaction.amount;

        let client_revision = client.with_patch(model::ClientPatch {
            available: Some(available),
            ..Default::default()
        });

        return self.sanitize(self.store.process_transaction(&client_revision, &transaction).await);
    }

    pub async fn withdrawal(&self, client: model::Client, transaction: model::Transaction) -> SanitizedResult<()> {
        let available = client.available - transaction.amount;
        if available.is_sign_negative() {
            return Err(SanitizedError::UserError("not enough funds available".to_string()));
        }

        let client_revision = client.with_patch(model::ClientPatch {
            available: Some(available),
            ..Default::default()
        });

        return self.sanitize(self.store.process_transaction(&client_revision, &transaction).await);
    }

    pub async fn dispute(&self, client: model::Client, dispute_tx: model::Transaction) -> SanitizedResult<()> {
        let reference_tx = match self.get_transaction_by_id(dispute_tx.tx).await? {
            Some(tx) => tx,
            None => {
                return Err(SanitizedError::UserError("dispute referenced tx does not exist, skipping".to_string()));
            }
        };

        if let Some(d) = self.get_dispute_by_reference_tx_id(dispute_tx.tx).await? {
            if !d.is_deleted {
                return Err(SanitizedError::UserError("tx is already under dispute, skipping".to_string()));
            }
        }

        let available = client.available - reference_tx.amount;
        let held = client.held + reference_tx.amount;
        let client_revision = client.with_patch(model::ClientPatch {
            available: Some(available),
            held: Some(held),
            ..Default::default()
        });

        let dispute = model::Dispute::new(reference_tx);

        return self.sanitize(self.store.process_dispute(&client_revision, &dispute).await);
    }

    pub async fn resolve(&self, client: model::Client, resolve_tx: model::Transaction) -> SanitizedResult<()> {
        let dispute = match self.get_dispute_by_reference_tx_id(resolve_tx.tx).await? {
            Some(d) => {
                if d.is_deleted {
                    return Err(SanitizedError::UserError("resolve tx is no longer disputed, skipping".to_string()));
                }
                d
            }
            None => {
                return Err(SanitizedError::UserError("resolve tx is not disputed, skipping".to_string()));
            }
        };

        let available = client.available + dispute.referenced_tx.amount;
        let held = client.held - dispute.referenced_tx.amount;

        let client_revision = client.with_patch(model::ClientPatch {
            available: Some(available),
            held: Some(held),
            ..Default::default()
        });
        let dispute_revision = dispute.with_patch(true);

        return self.sanitize(self.store.remove_dispute(&client_revision, &dispute_revision).await);
    }

    pub async fn chargeback(&self, client: model::Client, chargeback_tx: model::Transaction) -> SanitizedResult<()> {
        let dispute = match self.get_dispute_by_reference_tx_id(chargeback_tx.tx).await? {
            Some(d) => {
                if d.is_deleted {
                    return Err(SanitizedError::UserError("chargeback tx is no longer disputed, skipping".to_string()));
                }
                d
            }
            None => {
                return Err(SanitizedError::UserError("chargeback tx is not disputed, skipping".to_string()));
            }
        };

        let held = client.held - dispute.referenced_tx.amount;

        let client_revision = client.with_patch(model::ClientPatch {
            held: Some(held),
            locked: Some(true),
            ..Default::default()
        });
        let dispute_revision = dispute.with_patch(true);

        return self.sanitize(self.store.remove_dispute(&client_revision, &dispute_revision).await);
    }
}
