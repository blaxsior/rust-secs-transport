use secs_ii::item::Secs2Variant;

use crate::transport::{
    SecsTransport,
    error::SecsTransportError,
    secs1::{
        block::{Secs1Block, Secs1BlockHeader},
        config::Secs1TransportConfig,
        link::Secs1Link,
    },
};

///
/// Secs-I 통신 구현 (Block <-> Message)
///
pub struct Secs1Transport<Link: Secs1Link> {
    link: Link,
    secs_config: Secs1TransportConfig,
}

impl<Link: Secs1Link> Secs1Transport<Link> {
    pub fn new(link: Link, secs_config: Secs1TransportConfig) -> Result<Self, SecsTransportError> {
        // X_Y_Z = Z에 대해 Y인 채널에 대해 X 수행

        Ok(Self { link, secs_config })
    }

    /// before item이 기대하고 있는 아이템이 맞는지 체크
    fn is_expected(before: &Secs1BlockHeader, after: &Secs1BlockHeader) -> bool {
        // r bit / deviceID / system bytes / w bit / message id  동일
        // block no는 1 증가

        return before.rbit == after.rbit
            && before.device_id == after.device_id
            && before.system_bytes == after.system_bytes
            && before.wbit == after.wbit
            && before.block_no.wrapping_add(1) == after.block_no;
    }

    fn handle_primary(buffer: &mut Vec<Secs1Block>, block: Secs1Block) {
        if(block.header.is_end()) {
            // MESSAGE COMPLETE
            if(block.header.need_reply()) {
                // SAVE SYSTEM BYTES FOR REPLY
            }
        } else {
            // SET INTER-BLOCK TIMER(T4)
            // SET EXPECTED BLOCK
        }
    }

    
    fn handle_secondary(buffer: &mut Vec<Secs1Block>, block: Secs1Block) {
        if(block.header.is_end()) {
            // MESSAGE COMPLETE -> BREAK LOOP
        } else {
            // SET INTER-BLOCK TIMER(T4)
            // SET EXPECTED BLOCK
        }
    }
}

impl<Link: Secs1Link> SecsTransport for Secs1Transport<Link> {
    /// 데이터를 합친 후 전송
    async fn send(&mut self, item: secs_ii::item::Secs2Variant) -> Result<(), SecsTransportError> {
        // 01. item to blocks
        let blocks: Vec<Secs1Block> = vec![]; // item to blocks
        // 02. for blocks -> send and wait for success response (ACK/NAK)
        for block in blocks {
            // 03. err occured -> Err(SecsTransportError)
            self.link
                .send(block)
                .await
                .map_err(|_| SecsTransportError::SendFailed)?;
        }
        // 04. success case -> Ok(())
        Ok(())
    }

    /// message protocol receive에 대한 구현
    async fn recv(&mut self) -> Result<Secs2Variant, SecsTransportError> {
        // 블록 번호 -> 단일 블록의 경우 0 허용, 아니면 1부터 시작하여 1씩 증가 필요
        let mut before_header: Option<Secs1BlockHeader> = None;
        let mut blocks: Vec<Secs1Block> = Vec::new();

        loop {
            let block = self.link.recv().await?;

            // KNOWN DEVICE -> BLOCK ERROR
            if block.header.device_id != self.secs_config.device_id {
                return Err(SecsTransportError::BlockInvalid);
            }

            // Duplicate -> just discard Block
            if let Some(header_info) = &before_header {
                if header_info == &block.header {
                    continue;
                }
            }

            let is_expected = if let Some(header_info) = &before_header {
                Self::is_expected(header_info, &block.header)
            } else {
                false
            };

            if !is_expected {
                // 기대한 블록이 아님. primary first case만 허용
                if !block.header.is_primary() || !block.header.is_first_block() {
                    return Err(SecsTransportError::BlockInvalid);
                }
                // FIRST BLOCK OF PRIMARY MESSAGE
                Self::handle_primary(&mut blocks, block);
            } else {
                if block.header.is_primary() {
                    // cancel inter block timer(t4)
                   Self::handle_primary(&mut blocks, block);
                } else {
                    // is secondary
                    if block.header.is_first_block() {
                        // cancel reply block timer(t3)
                    } else {
                        // cancel reply timer(t4)
                    }
                    Self::handle_secondary(&mut blocks, block);
                }
            }
        }

        Ok(Secs2Variant::Char)
    }
}
