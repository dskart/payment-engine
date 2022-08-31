use crate::store;
use simple_error::SimpleError;

#[derive(Default)]
pub struct Conditionals(Vec<(keyvaluestore::ConditionalResult, store::Error)>);

impl Conditionals {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, r: keyvaluestore::ConditionalResult, err: store::Error) {
        self.0.push((r, err))
    }

    pub fn error(self) -> store::Error {
        self.0
            .into_iter()
            .find_map(|(c, e)| if c.failed() { Some(e) } else { None })
            .unwrap_or_else(|| store::Error::Other(Box::new(SimpleError::new("unknown conditional failure"))))
    }
}
