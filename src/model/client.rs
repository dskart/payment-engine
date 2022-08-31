use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Client {
    pub id: u16,
    pub available: f32,
    pub held: f32,
    pub total: f32,
    pub locked: bool,

    pub creation_time: DateTime<Utc>,
    pub revision_number: u32,
    pub revision_time: DateTime<Utc>,
}

#[derive(Debug, Default, PartialOrd, Serialize, Deserialize)]
pub struct CSVClient {
    pub client: u16,
    pub available: f32,
    pub held: f32,
    pub total: f32,
    pub locked: bool,
}

impl Ord for CSVClient {
    fn cmp(&self, other: &Self) -> Ordering {
        self.client.cmp(&other.client)
    }
}

impl PartialEq for CSVClient {
    fn eq(&self, other: &Self) -> bool {
        (self.client, self.available, self.held, self.total, self.locked) == (other.client, other.available, other.held, other.total, other.locked)
    }
}

impl Eq for CSVClient {}

impl From<Client> for CSVClient {
    fn from(c: Client) -> Self {
        return Self {
            client: c.id,
            available: c.available,
            held: c.held,
            total: c.total,
            locked: c.locked,
        };
    }
}

#[derive(Clone, Default, Debug)]
pub struct ClientPatch {
    pub available: Option<f32>,
    pub held: Option<f32>,
    pub locked: Option<bool>,
}

impl Client {
    pub fn new(id: u16, available: Option<f32>) -> Self {
        let now = Utc::now();
        let mut ret = Client {
            id,
            locked: false,
            creation_time: now,
            revision_number: 1,
            revision_time: now,
            ..Default::default()
        };

        if let Some(v) = available {
            ret.available = v;
        }

        ret.total = ret.available + ret.held;

        return ret;
    }

    pub fn with_patch(mut self, p: ClientPatch) -> Self {
        self.revision_number += 1;
        self.revision_time = Utc::now();
        if let Some(available) = p.available {
            self.available = available;
        }
        if let Some(held) = p.held {
            self.held = held;
        }
        if let Some(locked) = p.locked {
            self.locked = locked;
        }

        self.total = self.available + self.held;

        return self;
    }
}
