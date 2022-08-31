use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CSVTransaction {
    #[serde(rename = "type")]
    pub record_type: RecordType,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub record_type: RecordType,
    pub client: u16,
    pub tx: u32,
    pub amount: f32,

    pub creation_time: DateTime<Utc>,
    pub revision_number: u32,
    pub revision_time: DateTime<Utc>,
}

#[derive(Clone, Default, Debug)]
pub struct TransactionPatch {
    pub is_disputed: Option<bool>,
}

impl From<CSVTransaction> for Transaction {
    fn from(csv_tx: CSVTransaction) -> Self {
        let now = Utc::now();
        let amount = match csv_tx.amount {
            Some(x) => x,
            None => 0.0,
        };

        return Transaction {
            record_type: csv_tx.record_type,
            client: csv_tx.client,
            tx: csv_tx.tx,
            amount,

            creation_time: now,
            revision_number: 1,
            revision_time: now,
        };
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RecordType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}
