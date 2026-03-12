use secs_ii::item::Secs2Variant;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

///
/// SecsTransport 처리 시 예외
///
#[derive(Error, Debug)]
pub enum SecsTransportError {
    #[error("failed to connect")]
    ConnectionFailed(#[source]Option<Box<dyn std::error::Error + Send + Sync>>),

    #[error("failed to send message")]
    SendFailed(#[source] SendError<Secs2Variant>),

    #[error("failed to receive message")]
    RecvFailed,

    #[error("connection closed")]
    ConnectionClosed
}