use rand::RngCore;
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::ops::Deref;

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct Id(Vec<u8>);

pub const ID_LENGTH: usize = 20;

impl Id {
    pub fn generate() -> Id {
        let mut id = Vec::new();
        id.resize(ID_LENGTH, 0u8);
        rand::thread_rng().fill_bytes(&mut id);
        Id(id)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for Id {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl From<u32> for Id {
    fn from(v: u32) -> Self {
        let mut id = v.to_be_bytes().to_vec();
        id.resize(ID_LENGTH, 0u8);
        Self(id)
    }
}

impl AsRef<[u8]> for Id {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Id> for Vec<u8> {
    fn from(id: Id) -> Self {
        id.0
    }
}

// impl From<Id> for u32 {
//     fn from(id: Id) -> Self {
//         u32::from_be_bytes(id.as_bytes())
//     }
// }

impl Deref for Id {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Id, D::Error> {
        Ok(Id(Vec::<u8>::deserialize(deserializer)?))
    }
}

impl Serialize for Id {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}
