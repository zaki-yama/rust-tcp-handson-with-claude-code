// Step 5: データ送受信とシーケンス番号管理
// RFC 9293 Section 3.4 (Sequence Numbers), Section 3.7 (Segmentation), Section 3.8 (Data Communication)

use std::cmp::Ordering;
use std::collections::BTreeMap;

// =============================================================================
// Phase A: シーケンス番号の定義
// =============================================================================

/// Task A1: SequenceNumber構造体の定義
/// 32ビットのシーケンス番号をラップする構造体
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequenceNumber(u32);

impl SequenceNumber {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u32 {
        self.0
    }

    /// Task A2: シーケンス番号の加算
    pub fn wrapping_add(&self, n: u32) -> Self {
        // TODO: Task A2 - ラップアラウンドを考慮した加算
        // ヒント: u32::wrapping_add を使用
        todo!("Task A2: Implement wrapping_add")
    }

    /// Task A2: シーケンス番号の減算
    pub fn wrapping_sub(&self, other: Self) -> u32 {
        // TODO: Task A2 - ラップアラウンドを考慮した減算
        // ヒント: u32::wrapping_sub を使用
        todo!("Task A2: Implement wrapping_sub")
    }

    /// Task A3: シーケンス番号の比較
    /// RFC 793のシーケンス番号比較アルゴリズムを実装
    pub fn compare(&self, other: Self) -> Ordering {
        // TODO: Task A3 - RFC 793準拠の比較
        // ヒント: 2^31 (約21億) を超える差は逆方向と判定
        // (i < j) = ((i < j) && (j - i < 2^31)) || ((i > j) && (i - j > 2^31))
        todo!("Task A3: Implement RFC 793 sequence number comparison")
    }
}

impl PartialOrd for SequenceNumber {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // TODO: Task A3 - compare メソッドを使用
        todo!("Task A3: Implement PartialOrd")
    }
}

impl Ord for SequenceNumber {
    fn cmp(&self, other: &Self) -> Ordering {
        // TODO: Task A3 - compare メソッドを使用
        todo!("Task A3: Implement Ord")
    }
}

/// Task A4: ISN生成
/// Initial Sequence Number（ISN）を生成
pub fn generate_isn() -> SequenceNumber {
    // TODO: Task A4 - ISN生成
    // ヒント: SystemTime::now() と UNIX_EPOCH を使用
    // RFC 6528推奨: 時刻ベース + ランダム性
    todo!("Task A4: Implement ISN generation")
}

// =============================================================================
// Phase B: 送信バッファの実装
// =============================================================================

/// Task B1: SendBuffer構造体の定義
pub struct SendBuffer {
    buffer: Vec<u8>,             // 送信待ちデータ
    next_seq: SequenceNumber,    // 次に送信するシーケンス番号
    unacked_seq: SequenceNumber, // 未確認の最小シーケンス番号
    max_buffer_size: usize,      // バッファの最大サイズ
}

impl SendBuffer {
    /// Task B2: 新しい送信バッファを作成
    pub fn new(initial_seq: SequenceNumber) -> Self {
        // TODO: Task B2 - SendBufferを初期化
        // ヒント: max_buffer_size は 64KB (65536) が標準的
        todo!("Task B2: Implement SendBuffer::new")
    }

    /// Task B2: データをバッファに追加
    pub fn write(&mut self, data: &[u8]) -> Result<usize, String> {
        // TODO: Task B2 - データを送信バッファに追加
        // ヒント:
        // 1. 利用可能な容量をチェック
        // 2. データをbufferの末尾に追加
        // 3. 追加したバイト数を返す
        todo!("Task B2: Implement write")
    }

    /// Task B2: 送信可能な残り容量
    pub fn available_space(&self) -> usize {
        // TODO: Task B2 - 空き容量を計算
        // ヒント: max_buffer_size - buffer.len()
        todo!("Task B2: Implement available_space")
    }

    /// Task B3: 次に送信するデータを取得（削除しない）
    pub fn peek(&self, size: usize) -> &[u8] {
        // TODO: Task B3 - 送信すべきデータを取得
        // ヒント: buffer[..min(size, buffer.len())]
        todo!("Task B3: Implement peek")
    }

    /// Task B3: 送信済みデータをマーク
    pub fn consume(&mut self, size: usize) {
        // TODO: Task B3 - 送信完了後の処理
        // ヒント: next_seq を size 分進める
        todo!("Task B3: Implement consume")
    }

    /// Task B4: ACK受信時の処理
    pub fn acknowledge(&mut self, ack_seq: SequenceNumber) -> Result<usize, String> {
        // TODO: Task B4 - 確認済みデータをバッファから削除
        // ヒント:
        // 1. ack_seq と unacked_seq を比較
        // 2. ack_seq > unacked_seq なら処理
        // 3. 確認されたバイト数を計算
        // 4. bufferから該当データを削除
        // 5. unacked_seq を更新
        todo!("Task B4: Implement acknowledge")
    }

    /// Task B4: 未確認データ量
    pub fn unacked_data(&self) -> usize {
        // TODO: Task B4 - 送信済み未確認データ量を計算
        // ヒント: next_seq - unacked_seq
        todo!("Task B4: Implement unacked_data")
    }

    pub fn next_seq(&self) -> SequenceNumber {
        self.next_seq
    }

    pub fn unacked_seq(&self) -> SequenceNumber {
        self.unacked_seq
    }
}

// =============================================================================
// Phase C: 受信バッファの実装
// =============================================================================

/// Task C1: ReceiveBuffer構造体の定義
pub struct ReceiveBuffer {
    buffer: Vec<u8>,                                 // 順序通りに並んだデータ
    next_expected: SequenceNumber,                   // 次に期待するシーケンス番号
    out_of_order: BTreeMap<SequenceNumber, Vec<u8>>, // 順序外データ
}

impl ReceiveBuffer {
    /// Task C2: 新しい受信バッファを作成
    pub fn new(initial_seq: SequenceNumber) -> Self {
        // TODO: Task C2 - ReceiveBufferを初期化
        todo!("Task C2: Implement ReceiveBuffer::new")
    }

    /// Task C2: データを受信
    pub fn receive(&mut self, seq: SequenceNumber, data: &[u8]) -> Result<(), String> {
        // TODO: Task C2 - 受信データを適切に格納
        // ヒント:
        // 1. seq と next_expected を比較
        // 2. seq == next_expected: 順序通り → bufferに追加
        // 3. seq > next_expected: 順序外 → out_of_orderに保管
        // 4. seq < next_expected: 重複 → 無視
        // 5. process_out_of_order() を呼び出し
        todo!("Task C2: Implement receive")
    }

    /// Task C3: データを読み取る
    pub fn read(&mut self, size: usize) -> Vec<u8> {
        // TODO: Task C3 - bufferから最大sizeバイトを取り出す
        // ヒント:
        // 1. buffer.drain(..min(size, buffer.len()))
        // 2. 取り出したデータを返す
        todo!("Task C3: Implement read")
    }

    /// Task C3: 読み取り可能なデータ量
    pub fn available(&self) -> usize {
        // TODO: Task C3 - 利用可能なデータ量を返す
        todo!("Task C3: Implement available")
    }

    /// Task C4: 順序外データの処理
    fn process_out_of_order(&mut self) {
        // TODO: Task C4 - out_of_orderから連続データをbufferに移動
        // ヒント:
        // 1. out_of_order を走査
        // 2. next_expected と一致するデータを探す
        // 3. 見つかったらbufferに追加
        // 4. next_expected を更新
        // 5. 再帰的に処理（連続データがなくなるまで）
        todo!("Task C4: Implement process_out_of_order")
    }

    /// Task C4: データにギャップがあるか確認
    pub fn has_gap(&self) -> bool {
        // TODO: Task C4 - 順序外データの有無を返す
        todo!("Task C4: Implement has_gap")
    }

    pub fn next_expected(&self) -> SequenceNumber {
        self.next_expected
    }
}

// =============================================================================
// Phase D: セグメント化ロジック
// =============================================================================

/// Task D1: セグメント構造の定義
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub seq: SequenceNumber,
    pub data: Vec<u8>,
}

impl Segment {
    pub fn new(seq: SequenceNumber, data: Vec<u8>) -> Self {
        // TODO: Task D1 - Segmentを生成
        todo!("Task D1: Implement Segment::new")
    }

    pub fn len(&self) -> usize {
        // TODO: Task D1 - データ長を返す
        todo!("Task D1: Implement Segment::len")
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Task D2: MSSベースのセグメント化
/// Ethernet MTU 1500 - IP header 20 - TCP header 20 = 1460
pub const DEFAULT_MSS: usize = 1460;

pub fn segment_data(data: &[u8], mss: usize, start_seq: SequenceNumber) -> Vec<Segment> {
    // TODO: Task D2 - データをmssサイズのセグメントに分割
    // ヒント:
    // 1. data.chunks(mss) で分割
    // 2. 各チャンクをSegmentに変換
    // 3. 連続したシーケンス番号を付与
    todo!("Task D2: Implement segment_data")
}

/// Task D3: セグメント検証
pub fn validate_segment(seg: &Segment, expected_seq: SequenceNumber, window_size: u32) -> bool {
    // TODO: Task D3 - セグメントが受信ウィンドウ内にあるか確認
    // ヒント:
    // 1. expected_seq <= seg.seq < expected_seq + window_size
    // 2. ラップアラウンドを考慮した比較
    todo!("Task D3: Implement validate_segment")
}

// =============================================================================
// Phase E: データ送受信統合
// =============================================================================

// Step04の状態マシンを再利用
// use crate::step04::{TcpStateMachine, TcpState, TcpEvent};
// 注意: この段階では状態マシンの統合は行わず、バッファ管理に集中

/// Task E1: TcpConnection構造体の定義
pub struct TcpConnection {
    // state: TcpStateMachine,  // 将来の統合用（Step04）
    send_buffer: SendBuffer,
    recv_buffer: ReceiveBuffer,
    local_seq: SequenceNumber,
    remote_seq: SequenceNumber,
}

impl TcpConnection {
    pub fn new(local_isn: SequenceNumber, remote_isn: SequenceNumber) -> Self {
        // TODO: Task E1 - TcpConnectionを初期化
        todo!("Task E1: Implement TcpConnection::new")
    }

    /// Task E2: データを送信バッファに書き込み
    pub fn send(&mut self, data: &[u8]) -> Result<usize, String> {
        // TODO: Task E2 - データを送信バッファに追加
        // ヒント:
        // 1. 状態チェック（現段階では省略可）
        // 2. send_buffer.write(data)
        todo!("Task E2: Implement send")
    }

    /// Task E3: セグメントを受信
    pub fn receive(&mut self, seq: SequenceNumber, data: &[u8]) -> Result<(), String> {
        // TODO: Task E3 - 受信セグメントを処理
        // ヒント:
        // 1. 状態チェック（現段階では省略可）
        // 2. validate_segment() でセグメント検証
        // 3. recv_buffer.receive()
        todo!("Task E3: Implement receive")
    }

    /// Task E3: データを読み取る
    pub fn read(&mut self, size: usize) -> Vec<u8> {
        // TODO: Task E3 - アプリケーションがデータを読み取る
        todo!("Task E3: Implement read")
    }

    /// Task E4: ACKを生成
    pub fn generate_ack(&self) -> SequenceNumber {
        // TODO: Task E4 - 次に期待するシーケンス番号を返す
        todo!("Task E4: Implement generate_ack")
    }

    /// Task E4: ACKを処理
    pub fn process_ack(&mut self, ack_seq: SequenceNumber) -> Result<(), String> {
        // TODO: Task E4 - ACK受信時の処理
        // ヒント: send_buffer.acknowledge(ack_seq)
        todo!("Task E4: Implement process_ack")
    }

    pub fn send_buffer(&self) -> &SendBuffer {
        &self.send_buffer
    }

    pub fn recv_buffer(&self) -> &ReceiveBuffer {
        &self.recv_buffer
    }
}

// =============================================================================
// デモ用main関数
// =============================================================================

fn main() {
    println!("Step 5: データ送受信とシーケンス番号管理");
    println!("===========================================");
    println!();
    println!("このステップでは、TCPのデータ転送機能を実装します：");
    println!("- Phase A: シーケンス番号の定義");
    println!("- Phase B: 送信バッファの実装");
    println!("- Phase C: 受信バッファの実装");
    println!("- Phase D: セグメント化ロジック");
    println!("- Phase E: データ送受信統合");
    println!("- Phase F: 統合テストと検証");
    println!();
    println!("実装を開始するには、各PhaseのTODOを順に実装してください。");
    println!("TDDアプローチ: cargo test phase_a_tests で各フェーズのテストを実行");
}

#[cfg(test)]
mod tests;
