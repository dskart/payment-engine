use crate::model::Id;

pub trait AsKey {
    fn as_key(&self) -> Vec<u8>;
}

impl AsKey for str {
    fn as_key(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl AsKey for Vec<u8> {
    fn as_key(&self) -> Vec<u8> {
        self.clone()
    }
}

impl AsKey for &[u8] {
    fn as_key(&self) -> Vec<u8> {
        self.to_vec()
    }
}

impl AsKey for keyvaluestore::Value {
    fn as_key(&self) -> Vec<u8> {
        self.to_vec()
    }
}

impl AsKey for u32 {
    fn as_key(&self) -> Vec<u8> {
        self.to_string().as_bytes().to_vec()
    }
}

impl AsKey for u16 {
    fn as_key(&self) -> Vec<u8> {
        self.to_string().as_bytes().to_vec()
    }
}

impl AsKey for Id {
    fn as_key(&self) -> Vec<u8> {
        (**self).clone()
    }
}

#[macro_export]
macro_rules! store_key {
    ($e:expr) => {{
        use crate::store::store_key::AsKey;
        $e.as_key()
    }};
    ($e:expr, $($r:expr),+) => {
        [store_key!($e), store_key!($($r),+)].concat()
    };
}
