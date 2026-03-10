use futures::Stream;
use secs_ii::item::Secs2Variant;
use std::pin::Pin;

use crate::transport::error::SecsTransportError;

pub mod error;
pub mod secs1;

pub trait SecsTransport {
    fn open(&mut self) -> impl std::future::Future<Output = Result<(), SecsTransportError>> + Send; // 포트 열기 또는 소켓 Connect
    fn close(&mut self)
    -> impl std::future::Future<Output = Result<(), SecsTransportError>> + Send;

    ///
    /// 데이터를 stream 형식으로 수신한다.
    /// 
    fn recv(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<Secs2Variant, SecsTransportError>> + Send>>;

    ///
    /// 입력된 데이터를 전송한다.
    ///
    fn send(
        &mut self,
        item: Secs2Variant,
    ) -> impl std::future::Future<Output = Result<(), SecsTransportError>> + Send;
}
