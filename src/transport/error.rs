use futures::channel::mpsc::{RecvError, SendError};
use thiserror::Error;

///
/// SecsTransport 처리 시 예외
///
#[derive(Error, Debug)]
pub enum SecsTransportError {
    #[error("failed to connect")]
    ConnectionFailed(#[source]Option<Box<dyn std::error::Error + Send + Sync>>),

    #[error("failed to send message")]
    SendFailed(#[source] SendError),

    #[error("failed to receive message")]
    RecvFailed(#[source] RecvError),

    #[error("connection closed")]
    ConnectionClosed
}