use std::{collections::VecDeque, sync::Arc};

use futures::FutureExt;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, WriteHalf},
    sync::{Mutex, Notify, mpsc},
    task::JoinHandle,
};
use tokio_serial::{SerialPortBuilder, SerialPortBuilderExt, SerialStream};

use crate::transport::{
    ConnectionMode, error::SecsTransportError, secs1::{
        block::{Secs1Block, Secs1HandshakeCode},
        config::Secs1TransportConfig,
    }
};

///
/// SECS-I 통신 block transfer 수행 시 상태를 표현
///
enum Secs1LinkState {
    /// 초기 상태
    IDLE,
    /// 전송 방향 수립 중
    LINECONTROL,
    /// 데이터 전송 중
    SEND,
    /// 데이터 수신 중
    RECEIVE,
    /// 전송 후 종료 단계
    COMPLETION,
}

///
/// Secs-I Block transfer 수준의 통신을 구현. block 조립은 담당하지 않는다.
///
///
pub struct Secs1Link {
    notifier: Arc<Notify>,
    send_buffer: Arc<Mutex<VecDeque<Secs1Block>>>,
    receiver: mpsc::Receiver<Secs1Block>,
    task: JoinHandle<()>,
}

impl Secs1Link {
    pub fn new(
        config: Secs1TransportConfig,
        stream: SerialStream,
    ) -> Result<Self, SecsTransportError> {
        // 1. 통신 통로 및 알림 장치 준비
        let (tx_to_upper, rx_to_upper) = mpsc::channel(256);
        let notify = Arc::new(Notify::new());
        let send_buffer = Arc::new(Mutex::new(VecDeque::new()));

        // worker 객체 생성
        let mut worker = Secs1LinkWorker::new(
            config,
            stream,
            Arc::clone(&send_buffer),
            Arc::clone(&notify),
            tx_to_upper,
        );

        // 3. 백그라운드 루프 실행
        let task: JoinHandle<()> = tokio::spawn(async move {
            worker.run().await;
        });

        // 4. 외부 인터페이스 객체 리턴
        Ok(Self {
            send_buffer,
            notifier: notify,
            receiver: rx_to_upper,
            task,
        })
    }

    ///
    /// SECS-II 메시지 획득
    ///
    pub async fn recv(&mut self) -> Result<Secs1Block, SecsTransportError> {
        self.receiver
            .recv()
            .await
            .ok_or_else(|| SecsTransportError::RecvFailed)
    }

    ///
    /// SECS-I block 메시지를 전송한다.
    ///
    pub async fn send_all<Itr>(&mut self, blocks: Itr)
    where
        Itr: IntoIterator<Item = Secs1Block>,
    {
        let mut buf = self.send_buffer.lock().await;
        for block in blocks {
            buf.push_back(block);
        }
        // 데이터가 들어왔음을 알림
        self.notifier.notify_one();
    }
}

impl Drop for Secs1Link {
    fn drop(&mut self) {
        // task thread 함께 종료 시도
        self.task.abort();
        self.receiver.close();
    }
}

///
///  실제 통신을 처리하는 객체
///
struct Secs1LinkWorker {
    config: Secs1TransportConfig,
    /// serial로 데이터 쓰기위한 부분
    writer: WriteHalf<SerialStream>,
    // serial로부터 읽은 데이터를 보관
    reader: mpsc::Receiver<u8>,
    /// 상위에서 받은 데이터를 보관
    buffer: Arc<Mutex<VecDeque<Secs1Block>>>,
    notifier: Arc<Notify>,
    tx_to_upper: mpsc::Sender<Secs1Block>,
    state: Secs1LinkState,
    retry_count: u8,
    handle: JoinHandle<()>,
}

impl Secs1LinkWorker {
    pub fn new(
        config: Secs1TransportConfig,
        stream: SerialStream,
        buffer: Arc<Mutex<VecDeque<Secs1Block>>>,
        notifier: Arc<Notify>,
        tx_to_upper: mpsc::Sender<Secs1Block>,
    ) -> Self {
        // 1. 스트림을 읽기/쓰기로 분리
        let (mut reader, writer) = tokio::io::split(stream);

        // 2. 내부 수신 채널 생성
        let (tx, rx) = mpsc::channel(1024);

        // 3. 수신 전담 태스크 실행 (유실 방지 핵심)
        let handle = tokio::spawn(async move {
            loop {
                // 한 바이트씩 읽어서 채널로 전달
                match reader.read_u8().await {
                    Ok(byte) => {
                        if tx.send(byte).await.is_err() {
                            break; // Worker가 종료되면 채널이 닫혀 에러 발생 -> 태스크 종료
                        }
                    }
                    Err(_) => break, // 시리얼 포트 에러 등 발생 시 종료
                }
            }
        });

        Self {
            config,
            writer,     // 쓰기용만 소유
            reader: rx, // 채널 수신단 소유
            buffer,
            notifier,
            tx_to_upper, // 상위로 전달하기 위한 채널
            state: Secs1LinkState::IDLE,
            retry_count: 0,
            handle,
        }
    }
    pub async fn run(&mut self) {
        // state machine 처리
        loop {
            let result = match self.state {
                Secs1LinkState::IDLE => self.handle_idle().await,
                Secs1LinkState::LINECONTROL => self.handle_line_control().await,
                Secs1LinkState::RECEIVE => self.handle_receive().await,
                Secs1LinkState::SEND => self.handle_send().await,
                Secs1LinkState::COMPLETION => self.handle_completion().await,
            };

            if let Err(e) = result {
                // 연결이 끊기는 등 에러가 발생한 경우 종료
                // 추후 명세에 따라 개선 여지 있음
                break;
            }
        }
    }

    async fn has_remaining_data(&self) -> bool {
        let buf = self.buffer.lock().await;
        let has_data = !buf.is_empty();
        // 데이터가 남아 있는 상태에서 알람이 있는 상태였다면 알람을 초기화, 재시도를 막음
        if has_data {
            self.notifier.notified().now_or_never();
        }
        has_data
    }

    ///
    /// 데이터 전송을 위해 ENQ 신호를 보낸다.
    ///
    async fn send_enq(&mut self) -> Result<(), SecsTransportError> {
        self.state = Secs1LinkState::LINECONTROL; // LINECONTROL 모드로 전이
        self.retry_count = 0; // T2에 대한 재시도 카운트 초기화

        self.writer
            .write_u8(Secs1HandshakeCode::ENQ.into())
            .await
            .map_err(|_| SecsTransportError::SendFailed)
    }

    ///
    /// ENQ 신호 수신 후 수신 모드로 전환한다. EOT 신호의 전달이 필요.
    ///
    async fn switch_to_receive(&mut self) -> Result<(), SecsTransportError> {
        self.state = Secs1LinkState::RECEIVE; // RECEIVE 모드로 전이

        self.writer
            .write_u8(Secs1HandshakeCode::EOT.into())
            .await
            .map_err(|e| SecsTransportError::SendFailed)
    }

    ///
    /// ENQ 신호 송신 후 EOT 신호를 밭아 SEND 모드로 전이한다. 
    ///
    fn switch_to_send(&mut self) {
        self.state = Secs1LinkState::SEND; // SEND 모드로 전이
    }

    ///
    /// idle state에서의 동작을 처리한다.
    ///
    async fn handle_idle(&mut self) -> Result<(), SecsTransportError> {
        if self.has_remaining_data().await {
            return self.send_enq().await;
        }

        tokio::select! {
            // block 수신 알림. handle_idle에 다시 진입하여 send_enq 시도(데이터 없는 경우 고려)
            _ = self.notifier.notified() => {
                Ok(())
            }
            // 수신 데이터: 채널에서 데이터를 꺼낸다.
            result = self.reader.recv() => {
                let byte = result.ok_or_else(|| SecsTransportError::RecvFailed)?;

                if let Ok(code) = Secs1HandshakeCode::try_from(byte) {
                    if code == Secs1HandshakeCode::ENQ {
                        return self.switch_to_receive().await;
                    }
                }
                // ENQ 이외의 신호라면 그냥 무시함
                Ok(())
            }
        }
    }

    ///
    /// t2 timeout 발생 시 설정 대응
    /// 
    fn handle_t2_timeout(&mut self) {
        self.retry_count += 1;

        // FAILED_TO_SEND case
        if self.retry_count >= self.config.t2_rty {
            self.state = Secs1LinkState::IDLE; // retry 횟수 초과 시 idle 상태로 복귀
            self.retry_count = 0; // retry 0으로 초기화(필수는 아니나, 오류 막기 위한 목적)
        }
    }

    ///
    /// line control 단계에서의 작업을 처리한다.
    ///
    async fn handle_line_control(&mut self) -> Result<(), SecsTransportError> {
        let t2_timeout = tokio::time::sleep(self.config.t2_timeout);
        tokio::pin!(t2_timeout); // select!에서 쓰기 위해 고정(pin)

        tokio::select! {
                // t2 timeout이 발생한 케이스
                _ = &mut t2_timeout => {
                    self.handle_t2_timeout();
                    Ok(())
                }

                // 데이터가 들어온 경우
                result = self.reader.recv() => {
                    let byte = result.ok_or_else(|| SecsTransportError::RecvFailed)?;

                    if let Ok(code) = Secs1HandshakeCode::try_from(byte) {
                        // 나는 passive인데 상대에게 ENQ 받음 -> 양보 
                        if code == Secs1HandshakeCode::ENQ && 
                            self.config.mode == ConnectionMode::Passive {
                            return self.switch_to_receive().await;
                        } else if code == Secs1HandshakeCode::EOT {
                            // EOT를 정상적으로 받아 SEND 모드로 전이
                            self.switch_to_send();
                            return Ok(());
                        }
                        // 이외 데이터는 버림
                    }
                    // ENQ 이외의 신호라면 그냥 무시함
                    Ok(())
                }
        }
    }

    async fn handle_receive(&self) -> Result<(), SecsTransportError> {
        todo!()
    }

    async fn handle_send(&self) -> Result<(), SecsTransportError> {
        todo!()
    }

    async fn handle_completion(&self) -> Result<(), SecsTransportError> {
        todo!()
    }
}

impl Drop for Secs1LinkWorker {
    fn drop(&mut self) {
        // 수신 태스크 종료 시도
        self.handle.abort();
    }
}

///
/// link 객체를 생성하기 위한 빌더
///
pub struct Secs1LinkBuilder {
    secs_config: Secs1TransportConfig,
    serial_config: SerialPortBuilder,
}

impl Secs1LinkBuilder {
    pub fn new(secs_config: Secs1TransportConfig, serial_config: SerialPortBuilder) -> Self {
        Self {
            secs_config,
            serial_config,
        }
    }

    pub fn build(self) -> Result<Secs1Link, SecsTransportError> {
        let serial_stream = self
            .serial_config
            .open_native_async()
            .map_err(|e| SecsTransportError::ConnectionFailed(Some(Box::new(e))))?;

        Secs1Link::new(self.secs_config, serial_stream)
    }
}
