use std::net::Ipv4Addr;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// Step2からTcpHeaderを再利用する必要があります
// プロジェクト構成に応じて適切にimportしてください
// use crate::step02::{TcpHeader, calculate_checksum_rfc1071};

// Raw socketの基本機能（Step1から再利用）
// 実装時にStep1のコードを参考にしてください

#[derive(Debug, Clone, PartialEq)]
pub enum TcpState {
    Closed,
    SynSent,
    Established,
}

#[derive(Debug)]
pub struct TcpConnection {
    socket_fd: i32,
    state: TcpState,
    local_seq: u32,  // 自分のシーケンス番号
    remote_seq: u32, // 相手のシーケンス番号
    local_port: u16,
    remote_ip: u32,
    remote_port: u16,
}

impl TcpConnection {
    fn new(remote_ip: u32, remote_port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        // Task B1: TcpConnection構造体の実装
        // - Raw socket作成
        // - 初期状態設定
        // - ローカルポート選択（動的割り当て）
        todo!("Task B1: TcpConnection::new() を実装してください")
    }

    fn connect(&mut self, timeout_secs: u64) -> Result<(), Box<dyn std::error::Error>> {
        // Task F1: 完全な3-way handshakeの実装
        // 1. SYN送信
        // 2. SYN-ACK受信・検証
        // 3. ACK送信
        // 4. 接続完了
        todo!("Task F1: 3-way handshake全体の流れを実装してください")
    }

    fn send_syn(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Task C2: SYN送信機能
        // - ISN生成
        // - SYNパケット作成
        // - 送信
        // - 状態変更 (CLOSED -> SYN-SENT)
        todo!("Task C2: SYN送信機能を実装してください")
    }

    fn create_syn_packet(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Task C1: SYNパケット構築
        // - SYNフラグ付きTCPヘッダー作成
        // - チェックサム計算
        todo!("Task C1: SYNパケット構築を実装してください")
    }

    fn receive_packet_timeout(
        &self,
        timeout_secs: u64,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Task D1: タイムアウト付きパケット受信
        // - ノンブロッキング受信のループ
        // - タイムアウト判定
        todo!("Task D1: タイムアウト付き受信を実装してください")
    }

    fn parse_received_packet(&self, data: &[u8]) -> Result<Vec<u8>, &'static str> {
        // Task D2: 受信パケット解析
        // - IPヘッダー長計算
        // - TCPヘッダー抽出・パース
        todo!("Task D2: 受信パケット解析を実装してください")
    }

    fn is_correct_syn_ack(&self, tcp_header: &[u8]) -> bool {
        // Task D3: SYN-ACK検証
        // - SYN + ACKフラグチェック
        // - ACK番号の正確性チェック (local_seq + 1)
        // - ポート番号チェック
        todo!("Task D3: SYN-ACK検証ロジックを実装してください")
    }

    fn send_ack(&mut self, ack_number: u32) -> Result<(), Box<dyn std::error::Error>> {
        // Task E2: ACK送信
        // - ACKパケット作成
        // - 送信
        // - 状態管理
        todo!("Task E2: ACK送信機能を実装してください")
    }

    fn create_ack_packet(&self, ack_number: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Task E1: ACKパケット構築
        // - ACKフラグ付きTCPヘッダー作成
        // - 正しいseq/ack番号設定
        todo!("Task E1: ACKパケット構築を実装してください")
    }

    fn complete_handshake(&mut self) {
        // Task E3: 接続確立完了
        // - 状態をESTABLISHEDに変更
        // - シーケンス番号更新
        todo!("Task E3: 接続確立完了処理を実装してください")
    }

    fn is_connected(&self) -> bool {
        // Task E4: 接続確認
        self.state == TcpState::Established
    }
}

fn generate_isn() -> u32 {
    // Task B2: ISN生成
    // RFC 6528推奨の安全な初期シーケンス番号生成
    // 簡易実装: 現在時刻 + ランダム値
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    // 注意: 本来はrand crateを使用しますが、簡易実装として現在時刻を使用
    // 実際の実装では crypto-secure randomを使用してください
    now.wrapping_add(12345) // 簡易的なランダム要素
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Step 3: 3-way Handshake Implementation");
    println!("========================================");

    // ログ初期化（必要に応じて）
    // env_logger::init();

    // デモ実行例
    println!("Demo: Attempting 3-way handshake with localhost:80");

    let remote_ip = u32::from_be_bytes([127, 0, 0, 1]); // localhost

    // 注意: 実際にはlocalhostの80番ポートにHTTPサーバーが動いている必要があります
    // テスト用にnetcatを使用: nc -l 80 （別ターミナルで実行）

    let mut conn = TcpConnection::new(remote_ip, 80)?;

    println!("Initial state: {:?}", conn.state);

    match conn.connect(5) {
        Ok(_) => {
            println!("✅ Successfully established TCP connection!");
            println!("Final state: {:?}", conn.state);
            println!("Connection details:");
            println!("  Local seq: {}", conn.local_seq);
            println!("  Remote seq: {}", conn.remote_seq);
        }
        Err(e) => {
            println!("❌ Connection failed: {}", e);
            println!("Current state: {:?}", conn.state);
        }
    }

    println!("\n📝 Next steps:");
    println!("1. 各Task（A1-F4）を順番に実装してください");
    println!("2. テストを実行して動作確認してください: cargo test");
    println!("3. Wiresharkでパケットキャプチャを確認してください");
    println!("4. 学習した内容をLEARNING_LOG.mdに記録してください");

    Ok(())
}

#[cfg(test)]
mod tests;
