use std::io::Read;

use secs_ii::item::Secs2Variant;

use crate::transport::error::SecsTransportError;

pub mod error;
pub mod secs1;

pub trait SecsTransport {
    async fn open(&mut self) -> Result<(), SecsTransportError>; // 포트 열기 또는 소켓 Connect
    async fn close(&mut self) -> Result<(), SecsTransportError>;

    ///
    /// byte 데이터를 받아 처리한다.
    ///
    async fn recv<R: Read + ?Sized>(&mut self, item: &R) -> Result<(), SecsTransportError>;

    ///
    /// 입력된 데이터를 전송한다.
    ///
    async fn send(&mut self, item: Secs2Variant) -> Result<(), SecsTransportError>;
}
