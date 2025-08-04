use serde::{Deserialize, Serialize};

pub trait SessionId:
    Serialize
    + for<'de> Deserialize<'de>
    + Clone
    + Eq
    + std::hash::Hash
    + std::fmt::Debug
    + std::fmt::Display
    + Send
    + Sync
    + 'static
{
    fn validate(&self) -> Result<(), SessionIdError>;
}
