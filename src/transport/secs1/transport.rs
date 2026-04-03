use secs_ii::item::Secs2Variant;

use crate::transport::{
    SecsTransport,
    error::{SecsTimeoutType, SecsTransportError},
    secs1::{
        block::{Secs1Block, Secs1BlockHeader},
        config::{DeviceId, Secs1TransportConfig},
        link::Secs1Link,
    },
};

/// 현재 진행 중인 트랜잭션 상태
pub enum TransactionState {
    Idle,
    /// Message 수신 중 (T4 대기)
    Receiving {
        expected_header: Secs1BlockHeader,
        blocks: Vec<u8>,
    },
    /// Primary 전송 후 Secondary Message 대기 중 (T3 대기)
    WaitingForReply {
        expected_header: Secs1BlockHeader,
    },
}

impl TransactionState {
    /// recv 모드로 진입한다.
    fn recv_start(&mut self, block: Secs1Block) {
        *self = TransactionState::Receiving {
            expected_header: block.header.clone(),
            blocks: block.data,
        };
    }

    /// recv 모드에서 아이템을 추가한다.
    fn recv_next(&mut self, block: Secs1Block) {
        if let TransactionState::Receiving {
            expected_header,
            blocks,
        } = self
        {
            *expected_header = block.header.clone();
            blocks.extend(block.data);
        }
    }

    fn is_complete(&self) -> bool {
        match self {
            TransactionState::Receiving {
                expected_header, ..
            } => expected_header.is_end(),
            _ => false,
        }
    }
}

///
/// Secs-I 통신 구현 (Block <-> Message)
///
pub struct Secs1Transport<Link: Secs1Link> {
    link: Link,
    secs_config: Secs1TransportConfig,
    /// 중복 수신 방지용 헤더 정보
    last_header: Option<Secs1BlockHeader>,
    /// 현재 트랜잭션 상태
    state: TransactionState,
    t3_timer: Option<tokio::time::Sleep>,
}

impl<Link: Secs1Link> Secs1Transport<Link> {
    pub fn new(link: Link, secs_config: Secs1TransportConfig) -> Result<Self, SecsTransportError> {
        // X_Y_Z = Z에 대해 Y인 채널에 대해 X 수행

        Ok(Self {
            link,
            secs_config,
            last_header: None,
            state: TransactionState::Idle,
            t3_timer: None,
        })
    }

    /// after item이 현재 기대하고 있는 아이템이 맞는지 체크
    // fn is_expected(before: &Secs1BlockHeader, after: &Secs1BlockHeader) -> bool {
    //     // r bit / deviceID / system bytes / w bit / message id  동일
    //     // block no는 1 증가

    //     return before.rbit == after.rbit
    //         && before.device_id == after.device_id
    //         && before.system_bytes == after.system_bytes
    //         && before.wbit == after.wbit
    //         && before.block_no.wrapping_add(1) == after.block_no;
    // }

    // /// after item이 현재 기대하고 있는 reply 아이템의 첫 헤더인지 체크
    // fn is_expected_reply(before: &Secs1BlockHeader, after: &Secs1BlockHeader) -> bool {
    //     // deviceID / system bytes / w bit / message id  동일
    //     // block no는 1 증가

    //     return before.rbit != after.rbit // r bit 반전
    //         && before.device_id == after.device_id // device id 동일
    //         && before.system_bytes == after.system_bytes // system bytes 동일
    //         && after.is_first_block() // first block of
    //         && !after.is_primary(); // secondary
    // }

    /// device id가 알려진 장치인지 체크
    fn is_known_device(&self, device_id: &DeviceId) -> bool {
        self.secs_config.device_id == *device_id
    }

    /// 헤더가 이전에 수신한 헤더와 동일한지 체크 (중복 수신 방지)
    fn is_duplicate(&self, header: &Secs1BlockHeader) -> bool {
        if let Some(last) = &self.last_header {
            return last == header;
        }
        return false;
    }

    /// 헤더가 현재 기대하는 헤더인지 체크
    fn is_expected(&self, header: &Secs1BlockHeader) -> bool {
        match &self.state {
            TransactionState::Receiving {
                expected_header, ..
            } => expected_header == header,
            TransactionState::WaitingForReply { expected_header } => expected_header == header,
            _ => false,
        }
    }

    fn is_primary_first_block(&self, header: &Secs1BlockHeader) -> bool {
        header.is_primary() && header.is_first_block()
    }

    fn reset_to_idle(&mut self) {
        self.t3_timer = None;
        self.state = TransactionState::Idle;
    }

    fn fail_with<T>(&mut self, err: SecsTransportError) -> Result<T, SecsTransportError> {
        self.reset_to_idle();
        Err(err)
    }

    async fn wait_and_recv_block(&mut self) -> Result<Secs1Block, SecsTransportError> {
        let t4_timer = match &self.state {
            TransactionState::Receiving { .. } => {
                Some(tokio::time::sleep(self.secs_config.t4_timeout))
            }
            _ => None,
        };
        tokio::pin!(t4_timer);

        tokio::select! {
            res = self.link.recv() => {
                res
            },
            _ = async { t4_timer.as_mut().as_pin_mut().unwrap().await }, if t4_timer.is_some() => {
                return self.fail_with(SecsTransportError::Timeout(SecsTimeoutType::T4));
            }
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
        loop {
            // t4 timeout 고려하여 아이템 받기
            let block = self.wait_and_recv_block().await?;

            // KNOWN DEVICE -> BLOCK ERROR
            if !self.is_known_device(&block.header.device_id) {
                return self.fail_with(SecsTransportError::UnknownDeviceId(block.header.device_id));
            }

            // Duplicate -> just discard Block
            if self.is_duplicate(&block.header) {
                continue;
            }

            // 기대한 블록인지 체크
            if !self.is_expected(&block.header) {
                // 기대한 블록이 아님. primary first case만 허용
                if !self.is_primary_first_block(&block.header) {
                    return self.fail_with(SecsTransportError::BlockInvalid);
                }
                // FIRST BLOCK OF PRIMARY MESSAGE
                // 새로운 Receiving 트랜잭션으로 간주, 기존 정보 제거
                self.state.recv_start(block);
            } else {
                // secondary first block -> reply timer 초기화,
                // secondary receive mode로 전환

                // else -> Receiving mode -> 아이템 추가
                match &mut self.state {
                    TransactionState::Idle => unreachable!(), // is_expected에서 걸러짐
                    TransactionState::Receiving { .. } => self.state.recv_next(block),
                    TransactionState::WaitingForReply { .. } => {
                        // reply timer 초기화
                        self.t3_timer = None;
                        // secondary receive mode로 전환
                        self.state.recv_start(block);
                    }
                }

                if self.state.is_complete() {
                    // complete -> message로 변환하여 반환
                    if let TransactionState::Receiving { blocks, .. } = &self.state {
                        // blocks to message 로직 처리
                        let item = Secs2Variant::try_from(blocks.as_slice())
                            .map_err(|_| SecsTransportError::RecvFailed)?;
                        return Ok(item);
                    }
                }
                // TODO: complete 아니면 대응되는 expected block build 필요
            }
        }
    }
}
