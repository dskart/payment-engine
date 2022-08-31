use crate::model;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dispute {
    pub id: model::Id,
    pub referenced_tx: model::Transaction,
    // This is the same thing as resolved and/or chargedback
    pub is_deleted: bool,

    pub creation_time: DateTime<Utc>,
    pub revision_number: u32,
    pub revision_time: DateTime<Utc>,
}

impl Dispute {
    pub fn new(referenced_tx: model::Transaction) -> Self {
        let now = Utc::now();
        return Dispute {
            id: model::Id::generate(),
            referenced_tx,
            is_deleted: false,
            creation_time: now,
            revision_number: 1,
            revision_time: now,
        };
    }

    pub fn with_patch(mut self, is_deleted: bool) -> Self {
        self.revision_number += 1;
        self.revision_time = Utc::now();
        self.is_deleted = is_deleted;
        return self;
    }
}
