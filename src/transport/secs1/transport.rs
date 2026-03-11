use std::sync::{Arc, Mutex};

use futures::{SinkExt, channel::mpsc};
use secs_ii::item::Secs2Variant;
use tokio::io::AsyncReadExt;
use tokio_serial::{SerialPortBuilderExt, SerialStream};

use crate::transport::{SecsTransport, error::SecsTransportError};

///
/// secs1message 전송을 다루는 구조체
///
pub struct Secs1Transport {
    // serial_port: SerialStream,
    receiver: mpsc::Receiver<Result<Secs2Variant, SecsTransportError>>,
    sender: mpsc::Sender<Secs2Variant>,
}

pub enum Secs1TransportState {
    Idle,
    WaitAck,
    SendingBlock,
    ReceivingBlock,
}

impl Secs1Transport {
    pub fn new(port_name: &str, baud_rate: u32) -> Result<Self, SecsTransportError> {
        let serial_port = tokio_serial::new(port_name, baud_rate)
            .open_native_async()
            .map_err(|e| SecsTransportError::ConnectionFailed(Some(Box::new(e))))?;

        let (tx_send, rx_send) = mpsc::channel::<Secs2Variant>(256);
        let (tx_recv, rx_recv) = mpsc::channel::<Result<Secs2Variant, SecsTransportError>>(256);

        tokio::spawn(Self::transport_impl(serial_port, rx_send, tx_recv));

        Ok(Self {
            receiver: rx_recv,
            sender: tx_send,
        })
    }

    // TODO: tokio select! 을 이용해 예시와 유사하게 개발 필요 https://github.com/tokio-rs/tokio/blob/master/examples/chat.rs
    async fn transport_impl(
        serial_port: SerialStream,
        mut rx_sender: mpsc::Receiver<Secs2Variant>,
        mut tx_receiver: mpsc::Sender<Result<secs_ii::item::Secs2Variant, SecsTransportError>>,
    ) -> Result<(), SecsTransportError> {
        const BUFFER_SIZE: usize = 256;
        let mut buf = [0u8; BUFFER_SIZE];

        // TODO: serial port 읽기 / 쓰기 로직 구현
        // tokio::select! {
        // }

        Err(SecsTransportError::ConnectionClosed)
    }
}

impl SecsTransport for Secs1Transport {
    async fn open(&mut self) -> Result<(), SecsTransportError> {
        Ok(())
    }

    async fn close(&mut self) -> Result<(), SecsTransportError> {
        todo!()
    }

    async fn send(&mut self, item: secs_ii::item::Secs2Variant) -> Result<(), SecsTransportError> {
        self.sender
            .send(item)
            .await
            .map_err(|e| SecsTransportError::SendFailed(e))
    }

    async fn recv(&mut self) -> Result<Secs2Variant, SecsTransportError> {
        self.receiver
            .recv()
            .await
            .map_err(|e| SecsTransportError::RecvFailed(e))?
    }
}
