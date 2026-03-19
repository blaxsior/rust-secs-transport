use secs_ii::item::Secs2Variant;

use crate::transport::{SecsTransport, error::SecsTransportError, secs1::{config::Secs1TransportConfig, link::{Secs1Link, Secs1LinkBuilder, Secs1LinkImpl}}};

///
/// Secs-I 통신 구현 (Block <-> Message)
///
pub struct Secs1Transport<Link: Secs1Link> {
    link: Link,
    secs_config: Secs1TransportConfig
}

impl<Link: Secs1Link> Secs1Transport<Link> {
    pub fn new(link: Link, secs_config: Secs1TransportConfig) -> Result<Self, SecsTransportError> {
        // X_Y_Z = Z에 대해 Y인 채널에 대해 X 수행

        Ok(Self {
            link,
            secs_config,
        })
    }
}

impl<Link: Secs1Link> SecsTransport for Secs1Transport<Link> {

    async fn send(&mut self, item: secs_ii::item::Secs2Variant) -> Result<(), SecsTransportError> {
        self.link.send_all(blocks).await;
        todo!();

    }

    async fn recv(&mut self) -> Result<Secs2Variant, SecsTransportError> {
        self.recv().await;
        todo!();
    }
}
