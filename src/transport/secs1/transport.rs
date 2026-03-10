use futures::{SinkExt, TryStreamExt, channel::mpsc};
use secs_ii::item::Secs2Variant;
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
        Ok(())
    }

    async fn close(&mut self) -> Result<(), SecsTransportError> {
        todo!()
    }

    async fn send(&mut self, item: secs_ii::item::Secs2Variant) -> Result<(), SecsTransportError> {
        todo!()
    }

    fn recv(
        &mut self,
    ) -> std::pin::Pin<
        Box<
            dyn futures::Stream<Item = Result<secs_ii::item::Secs2Variant, SecsTransportError>>
                + Send,
        >,
    > {
        let (mut tx, rx) = mpsc::channel::<Result<Secs2Variant, SecsTransportError>>(10);

        // 2. Producer task 생성
        tokio::spawn(async move {
            loop {
                let item = Ok(Secs2Variant::Jis8); // 예시 데이터
                if let Err(_) = tx.send(item).await {
                    break; // Receiver가 drop되면 종료
                }
            }
        });

        Box::pin(rx.into_stream())
    }
}
