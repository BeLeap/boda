use std::fmt;

pub enum Error {}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: update to custom implemented pretty print
        write!(f, "{:#?}", self)
    }
}
