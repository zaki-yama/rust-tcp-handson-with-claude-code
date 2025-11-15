// Step 4: TCP State Machine Implementation
//
// このステップでは、TCP状態マシンの完全な実装を行います。
// RFC 9293 Section 3.3.2 (State Machine Overview), Figure 5 (TCP Connection State Diagram)
// の状態遷移図に基づいて実装してください。

use std::fmt;

// =============================================================================
// Phase A: TCP状態の定義
// =============================================================================

// Task A1: TcpState列挙型の定義
// RFC 9293で定義されている11種類のTCP状態を実装してください
/// ref. RFC 9293 3.3.2. State Machine Overview
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
    Closed,
}

// Task A2: TcpEvent列挙型の定義
// 状態遷移のトリガーとなるイベントを定義してください
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TcpEvent {
    Connect,       // アプリケーションからの接続要求
    Listen,        // アプリケーションからの待ち受け要求
    ReceiveSyn,    // SYNパケット受信
    ReceiveSynAck, // SYN-ACKパケット受信
    ReceiveAck,    // ACKパケット受信
    ReceiveFin,    // FINパケット受信
    ReceiveRst,    // RSTパケット受信
    Close,         // アプリケーションからの切断要求
    Timeout,       // タイムアウト
}

// Task A3: StateMachineの基本構造
pub struct TcpStateMachine {
    current_state: TcpState,
    // 状態遷移履歴（デバッグ用）
    state_history: Vec<(TcpState, TcpState, TcpEvent)>,
}

impl TcpStateMachine {
    pub fn new() -> Self {
        Self {
            current_state: TcpState::Closed,
            state_history: vec![],
        }
    }

    pub fn current_state(&self) -> TcpState {
        self.current_state
    }
}

// =============================================================================
// Phase B: 状態遷移ロジック
// =============================================================================

impl TcpStateMachine {
    // Task B1: 状態遷移メソッドの実装
    pub fn transition(&mut self, event: TcpEvent) -> Result<TcpState, String> {
        let old_state = self.current_state;
        let new_state = self.next_state(old_state, event.clone())?;

        self.current_state = new_state;
        self.state_history.push((old_state, new_state, event));

        Ok(new_state)
    }

    // Task B2: 状態遷移テーブルの実装
    fn next_state(&self, current: TcpState, event: TcpEvent) -> Result<TcpState, String> {
        use TcpEvent::*;
        use TcpState::*;
        match (current, event) {
            // Closedからの遷移
            (Closed, Connect) => Ok(SynSent),
            (Closed, TcpEvent::Listen) => Ok(TcpState::Listen),

            // Listenからの遷移
            (TcpState::Listen, ReceiveSyn) => Ok(SynReceived),
            (TcpState::Listen, Close) => Ok(Closed),

            // SynSentからの遷移
            (SynSent, ReceiveSynAck) => Ok(Established),
            (SynSent, ReceiveSyn) => Ok(SynReceived),
            (SynSent, Close) => Ok(Closed),

            // SynReceivedからの遷移
            (SynReceived, ReceiveAck) => Ok(Established),
            (SynReceived, Close) => Ok(FinWait1),
            (SynReceived, ReceiveRst) => Ok(TcpState::Listen), // RFC 9293: パッシブオープン経由の場合はListenへ

            // Establishedからの遷移
            (Established, Close) => Ok(FinWait1),
            (Established, ReceiveFin) => Ok(CloseWait),

            // FinWait1からの遷移
            (FinWait1, ReceiveAck) => Ok(FinWait2),
            (FinWait1, ReceiveFin) => Ok(Closing),

            // FinWait2からの遷移
            (FinWait2, ReceiveFin) => Ok(TimeWait),

            // CloseWaitからの遷移
            (CloseWait, Close) => Ok(LastAck),

            // Closingからの遷移
            (Closing, ReceiveAck) => Ok(TimeWait),

            // LastAckからの遷移
            (LastAck, ReceiveAck) => Ok(Closed),

            // TimeWaitからの遷移
            (TimeWait, Timeout) => Ok(Closed),

            // RSTパケット処理（RFC 9293 Section 3.5.3）
            (Closed, ReceiveRst) => Ok(Closed), // 既にClosedの場合は何もしない
            (TcpState::Listen, ReceiveRst) => Ok(TcpState::Listen), // LISTENはRSTを無視
            (SynSent, ReceiveRst) => Ok(Closed),
            (Established, ReceiveRst) => Ok(Closed),
            (FinWait1, ReceiveRst) => Ok(Closed),
            (FinWait2, ReceiveRst) => Ok(Closed),
            (CloseWait, ReceiveRst) => Ok(Closed),
            (Closing, ReceiveRst) => Ok(Closed),
            (LastAck, ReceiveRst) => Ok(Closed),
            (TimeWait, ReceiveRst) => Ok(Closed),

            // 不正な遷移
            _ => Err(format!("Invalid transition: {:?} + {:?}", current, event)),
        }
    }

    // Task B3: 状態確認メソッド
    pub fn is_established(&self) -> bool {
        self.current_state == TcpState::Established
    }

    pub fn is_closed(&self) -> bool {
        self.current_state == TcpState::Closed
    }

    pub fn can_send_data(&self) -> bool {
        matches!(
            self.current_state,
            TcpState::Established | TcpState::CloseWait
        )
    }

    pub fn can_receive_data(&self) -> bool {
        matches!(
            self.current_state,
            TcpState::Established | TcpState::FinWait1 | TcpState::FinWait2
        )
    }
}

// =============================================================================
// Phase C: 接続確立時の状態管理
// =============================================================================

impl TcpStateMachine {
    // Task C1: アクティブオープンの実装
    // クライアント側: Closed → SynSent → Established
    pub fn active_open(&mut self) -> Result<(), String> {
        self.transition(TcpEvent::Connect)?;
        Ok(())
    }

    pub fn complete_active_open(&mut self) -> Result<(), String> {
        self.transition(TcpEvent::ReceiveSynAck)?;
        Ok(())
    }

    // Task C2: パッシブオープンの実装
    pub fn passive_open(&mut self) -> Result<(), String> {
        self.transition(TcpEvent::Listen)?;
        Ok(())
    }

    pub fn accept_connection(&mut self) -> Result<(), String> {
        self.transition(TcpEvent::ReceiveAck)?;
        Ok(())
    }

    // Task C3: 同時オープンの処理
    // SynSent → SynReceived
    pub fn simultaneous_open(&mut self) -> Result<(), String> {
        // SynSent状態でSYN受信 → SynReceived
        // 注: その後Establishedへ進むかは呼び出し側が判断
        self.transition(TcpEvent::ReceiveSyn)?;
        Ok(())
    }
}

// =============================================================================
// Phase D: 接続終了時の状態管理
// =============================================================================

impl TcpStateMachine {
    // Task D1: アクティブクローズの実装
    // Established → FinWait1 (→ FinWait2 → TimeWait → Closed)
    pub fn active_close(&mut self) -> Result<(), String> {
        self.transition(TcpEvent::Close)?;
        Ok(())
    }

    // Task D2: パッシブクローズの実装
    // Established → CloseWait (→ LastAck → Closed)
    pub fn passive_close(&mut self) -> Result<(), String> {
        self.transition(TcpEvent::ReceiveFin)?;
        Ok(())
    }

    // Task D3: 同時クローズの処理
    // (Established →) FinWait1 → Closing (→ TimeWait → Closed)
    pub fn simultaneous_close(&mut self) -> Result<(), String> {
        self.transition(TcpEvent::ReceiveFin)?;
        Ok(())
    }
}

// =============================================================================
// Phase E: エラーハンドリングと異常系
// =============================================================================

impl TcpStateMachine {
    // Task E1: RSTパケット処理
    // RFC 9293 Section 3.5.3: Reset Processing
    pub fn handle_reset(&mut self) -> Result<(), String> {
        // transition()経由でRST処理（状態遷移テーブルに従う）
        self.transition(TcpEvent::ReceiveRst)?;
        Ok(())
    }

    // Task E2: タイムアウト処理
    pub fn handle_timeout(&mut self) -> Result<(), String> {
        // TODO: 状態に応じたタイムアウト処理
        // - SynSent/SynReceived: Closedへ
        // - TimeWait: 2MSL後にClosedへ
        todo!()
    }

    // Task E3: 不正遷移の検出とログ
    pub fn get_state_history(&self) -> &[(TcpState, TcpState, TcpEvent)] {
        // TODO: 状態遷移履歴を返す
        todo!()
    }

    pub fn print_state_diagram(&self) {
        // TODO: 状態遷移履歴を見やすく表示
        // 形式: "1. Closed --[Connect]--> SynSent"
        todo!()
    }
}

// =============================================================================
// Display実装（デバッグ用）
// =============================================================================

impl fmt::Display for TcpState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for TcpEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// =============================================================================
// デモ用main関数
// =============================================================================

fn main() {
    println!("Step 4: TCP State Machine Implementation");
    println!("=========================================");
    println!();
    println!("このステップでは、TCP状態マシンを実装します。");
    println!("RFC 9293 Section 3.3.2, Figure 5 の状態遷移図を参照してください。");
    println!();
    println!("実装後、以下のコマンドでテストを実行してください:");
    println!("  cargo test --bin step04");
    println!();
    println!("各フェーズのTDD実行手順:");
    println!("  1. cargo test phase_a_tests");
    println!("  2. cargo test phase_b_tests");
    println!("  3. cargo test phase_c_tests");
    println!("  4. cargo test phase_d_tests");
    println!("  5. cargo test phase_e_tests");
    println!("  6. cargo test phase_f_tests");
}

#[cfg(test)]
mod tests;
