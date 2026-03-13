use secs_ii::item::Secs2Variant;

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
    ) -> impl Future<Output = Result<Secs2Variant, SecsTransportError>>;

    ///
    /// 입력된 데이터를 전송한다.
    ///
    fn send(
        &mut self,
        item: Secs2Variant,
    ) -> impl std::future::Future<Output = Result<(), SecsTransportError>> + Send;
}

///
/// SECS 통신 시 역할
/// 
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionMode {
    /// 요청을 시도하는 측 (= master / host)
    Active, // = Master
    /// 요청에 응답하는 측 (= slave / eqp)
    Passive
}