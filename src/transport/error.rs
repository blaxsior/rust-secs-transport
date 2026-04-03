use thiserror::Error;

use crate::transport::secs1::config::DeviceId;

///
/// SecsTransport 처리 시 예외
///
#[derive(Error, Debug)]
pub enum SecsTransportError {
    #[error("failed to connect")]
    ConnectionFailed(#[source]Option<Box<dyn std::error::Error + Send + Sync>>),

    #[error("failed to send message")]
    SendFailed,

    #[error("failed to receive message")]
    RecvFailed,

    #[error("connection closed")]
    ConnectionClosed,

    #[error("timeout: {0:?}")]
    Timeout(SecsTimeoutType),

    #[error("invalid block")]
    BlockInvalid,

    #[error("invalid block size {0}")]
    BlockSizeInvalid(u32),

    /// 보낼 아이템이 없음. 정말 특이한 상황
    #[error("no item to send")]
    NothingToSend,

    #[error("unknown device id: {0:?}")]
    UnknownDeviceId(DeviceId),
}

#[derive(Debug)]
pub enum SecsTimeoutType {
    T1,
    T2,
    T3,
    T4,
}