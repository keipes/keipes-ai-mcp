use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub mod cache;
// pub mod id;

#[derive(Debug, Error)]
pub enum SessionIdError {
    #[error("Encountered non-printable-ascii byte {0:#04X} in session id")]
    InvalidByte(u8),
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Invalid session id: {0}")]
    InvalidId(#[from] SessionIdError),
}

pub trait SessionId:
    Serialize + for<'de> Deserialize<'de> + Clone + Eq + std::hash::Hash + Send + Sync + 'static
{
}

// Session lives for the duration of a client's session.
// Session needs to be exposed to all router handles.
// Session is the only task which may read or write messages
// Session receives all messages not directed to a particular request
// Session is woken on incoming messages, then stops when done processing

// requests coming into the rmcp router get assigned individual tasks by the web server
// these tasks invoke async calls on the session to communicate over the network

// is session a transport?
pub trait Session: Send + Sync {
    // fn age(&self) -> u64;
    fn new() -> Self
    where
        Self: Sized;
    fn send() {}
    fn receive() {}
}

impl SessionId for String {}
impl Session for String {
    fn new() -> Self {
        Uuid::new_v4().to_string()
    }
}

pub struct UuidSession {
    id: String,
}

impl Session for UuidSession {
    fn new() -> Self
    where
        Self: Sized,
    {
        let id = Uuid::new_v4().to_string();
        Self { id }
    }
}

impl UuidSession {
    fn new(id: String) -> Self {
        Self { id }
    }
}
