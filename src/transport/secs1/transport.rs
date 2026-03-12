use secs_ii::item::Secs2Variant;
use tokio::{io::AsyncReadExt, sync::mpsc};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

use crate::transport::{SecsTransport, error::SecsTransportError};

///
/// Secs-I 통신 구현 (Block <-> Message)
///
pub struct Secs1Transport {
    // serial_port: SerialStream,
    receiver: mpsc::Receiver<Secs2Variant>,
    sender: mpsc::Sender<Secs2Variant>,
}

impl Secs1Transport {
    pub fn new(port_name: &str, baud_rate: u32) -> Result<Self, SecsTransportError> {
        let serial_port = tokio_serial::new(port_name, baud_rate)
            .open_native_async()
            .map_err(|e| SecsTransportError::ConnectionFailed(Some(Box::new(e))))?;

        // X_Y_Z = Z에 대해 Y인 채널에 대해 X 수행
        let (tx_from_serial, rx_from_serial) = mpsc::channel::<Secs2Variant>(256);
        let (tx_to_serial, rx_to_serial) = mpsc::channel::<Secs2Variant>(256);

        tokio::spawn(Self::transport_impl(
            serial_port,
            rx_to_serial,
            tx_from_serial,
        ));

        Ok(Self {
            receiver: rx_from_serial,
            sender: tx_to_serial,
        })
    }

    // TODO: tokio select! 을 이용해 예시와 유사하게 개발 필요 https://github.com/tokio-rs/tokio/blob/master/examples/chat.rs
    async fn transport_impl(
        mut serial_port: SerialStream,
        mut rx_to_serial: mpsc::Receiver<Secs2Variant>,
        mut tx_from_serial: mpsc::Sender<Secs2Variant>,
    ) {
        const BUFFER_SIZE: usize = 256;
        let mut buf = [0u8; BUFFER_SIZE];

        loop {
            // TODO: serial port 읽기 / 쓰기 로직 구현
            tokio::select! {
                // from serial port -> tx_
                result = serial_port.read(&mut buf) => {
                    match result {
                        Ok(0) => {
                            break;
                        },
                        Ok(n) => {
                            //작업 처리
                    tx_from_serial.send(Secs2Variant::Char);

                        }
                        // 에러 발생
                        Err(e) => {
                            break;
                        }
                    }
                }

                some = rx_to_serial.recv() => {
                    match some {
                        Some(data) => {
                            break;
                        },
                        None => todo!(),
                    }
                }
            }
        }
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
            .ok_or_else(|| SecsTransportError::RecvFailed)
    }
}
