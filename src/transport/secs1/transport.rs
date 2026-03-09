use tokio_serial::SerialPort;
use tokio_util::codec::Framed;

use crate::transport::{SecsTransport, error::SecsTransportError, secs1::codec::Secs1Codec};

///
/// secs1message 전송을 다루는 구조체
///
pub struct Secs1Transport {
    framed: Framed<Box<dyn SerialPort>, Secs1Codec>,
}

impl SecsTransport for Secs1Transport {
    async fn open(&mut self) -> Result<(), SecsTransportError> {
        todo!()
    }

    async fn close(&mut self) -> Result<(), SecsTransportError> {
        todo!()
    }

    async fn recv<R: std::io::Read + ?Sized>(
        &mut self,
        item: &R,
    ) -> Result<(), SecsTransportError> {
        todo!()
    }

    async fn send(&mut self, item: secs_ii::item::Secs2Variant) -> Result<(), SecsTransportError> {
        todo!()
    }
}
