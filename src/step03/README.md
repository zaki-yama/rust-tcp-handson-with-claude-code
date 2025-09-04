# Step 3: 3-way handshake の実装

## 🎯 このステップの最終目標
TCP接続確立の基本プロセス（3-way handshake）を実装し、実際のサーバーとの接続を確立する

## 📋 実装タスク一覧（推奨順序）

### Phase A: 3-way handshakeの理解と設計 (45分) ✅ 完了
- [x] **Task A1**: RFC 9293 Section 3.5を読んで3-way handshakeプロセスを理解
- [x] **Task A2**: シーケンス番号とACK番号の計算ルールを整理  
- [x] **Task A3**: 基本的な接続状態（CLOSED, SYN-SENT, ESTABLISHED）を定義
- [x] **Task A4**: プロジェクト構成を作成（main.rs, tests.rs, LEARNING_LOG.md）

### Phase B: 基本構造の実装 (60分) ✅ 完了
- [x] **Task B1**: `TcpConnection`構造体を定義（状態管理用）
- [x] **Task B2**: 初期シーケンス番号（ISN）生成機能を実装
- [x] **Task B3**: Step2の`TcpHeader`を再利用できるよう調整
- [x] **Task B4**: 基本的なパケット送信インフラを構築

### Phase C: SYNパケット送信機能 (45分) ✅ 完了
- [x] **Task C1**: SYNパケット構築機能を実装
- [x] **Task C2**: 指定ホスト・ポートへのSYN送信機能を実装
- [x] **Task C3**: 送信後の状態をCLOSED→SYN-SENTに変更
- [x] **Task C4**: SYN送信のテストを作成

### Phase D: SYN-ACKパケット受信処理 (60分)
- [ ] **Task D1**: パケット受信のためのタイムアウト付きrecv実装
- [ ] **Task D2**: 受信パケットのTCPヘッダー解析機能
- [ ] **Task D3**: SYN-ACKパケットの検証ロジック実装
- [ ] **Task D4**: ACK番号の正確性チェック機能

### Phase E: ACKパケット送信と接続完了 (45分)
- [ ] **Task E1**: SYN-ACK受信後のACKパケット構築
- [ ] **Task E2**: ACKパケット送信機能を実装
- [ ] **Task E3**: 状態をSYN-SENT→ESTABLISHEDに変更
- [ ] **Task E4**: 接続確立完了の確認機能

### Phase F: 統合テストと動作確認 (45分)
- [ ] **Task F1**: ローカルサーバーとの完全なhandshakeテスト
- [ ] **Task F2**: 実際のWebサーバー（例：httpbin.org:80）との接続テスト
- [ ] **Task F3**: Wiresharkでのパケットキャプチャ確認
- [ ] **Task F4**: エラーケース（タイムアウト、拒否）のテスト

---

## 📖 各Phaseの詳細ガイド

### Phase A: 3-way handshakeの理解と設計

#### Task A1: RFC理解
**何をする**: RFC 9293 Section 3.5の3-way handshake仕様を読む

**3-way handshakeの流れ**:
```
Client                           Server
CLOSED                           LISTEN
  |                                |
  | -- SYN seq=x -->              |
  |                               | 
SYN-SENT                          |
  |                              | -- SYN-ACK seq=y, ack=x+1 -->
  |                           SYN-RECEIVED
  | <-- SYN-ACK seq=y, ack=x+1 --|
  |                               |
  | -- ACK seq=x+1, ack=y+1 -->  |
  |                               |
ESTABLISHED                   ESTABLISHED
```

#### Task A2: シーケンス番号の計算
**何をする**: 各パケットのseq/ack番号計算ルールを整理

| パケット | seq番号 | ack番号 | 説明 |
|----------|---------|---------|------|
| SYN | ISN_client (x) | 0 | クライアントの初期シーケンス |
| SYN-ACK | ISN_server (y) | x+1 | サーバーの初期seq + クライアントseqをACK |
| ACK | x+1 | y+1 | 両方のISNをインクリメントしてACK |

#### Task A3: 基本状態定義
**何をする**: 最小限の接続状態を定義
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TcpState {
    Closed,
    SynSent,
    Established,
}
```

#### Task A4: プロジェクト構成
**何をする**: 必要なファイルとディレクトリを作成
```bash
src/step03/
├── main.rs          # メイン実装
├── README.md        # このファイル
├── tests.rs         # テストコード
└── LEARNING_LOG.md  # 学習ログ
```

---

### Phase B: 基本構造の実装

#### Task B1: TcpConnection構造体
**何をする**: 接続状態を管理する構造体を定義
```rust
#[derive(Debug)]
pub struct TcpConnection {
    socket_fd: i32,
    state: TcpState,
    local_seq: u32,     // 自分のシーケンス番号
    remote_seq: u32,    // 相手のシーケンス番号
    local_port: u16,
    remote_ip: u32,
    remote_port: u16,
}

impl TcpConnection {
    fn new(remote_ip: u32, remote_port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        // Raw socket作成
        // 初期状態設定
    }
}
```

#### Task B2: ISN生成
**何をする**: RFC準拠の初期シーケンス番号生成
```rust
use std::time::{SystemTime, UNIX_EPOCH};

fn generate_isn() -> u32 {
    // 現在時刻ベースの安全なISN生成
    // RFC 6528推奨の手法を簡易実装
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    now.wrapping_add(rand::random::<u32>())
}
```

#### Task B3: TcpHeader再利用
*何をする**: Step2のTcpHeaderを import して使用
```rust
// Step2のコードを参照可能にする
use crate::step02::{TcpHeader, calculate_checksum_rfc1071};
```

#### Task B4: パケット送信インフラ
**何をする**: IPパケット送信の基盤機能
```rust
impl TcpConnection {
    fn send_tcp_packet(&self, tcp_header: &TcpHeader, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        // Step1のIPヘッダー作成
        // TCPヘッダー + データの結合
        // Raw socket送信
    }
}
```

---

### Phase C: SYNパケット送信機能

#### Task C1: SYNパケット構築
**何をする**: SYNフラグ付きTCPヘッダーを作成
```rust
impl TcpConnection {
    fn create_syn_packet(&self) -> TcpHeader {
        let mut header = TcpHeader::new(
            self.local_port,
            self.remote_port,
            self.local_seq,     // ISN
            0,                  // ACK番号は0
            0x02,              // SYNフラグ（bit 1）
            8192,              // ウィンドウサイズ
        );
        
        // チェックサム計算
        header.calculate_checksum(
            u32::from_be_bytes([127, 0, 0, 1]), // 送信元IP
            self.remote_ip,
            &[]  // データなし
        );
        
        header
    }
}
```

#### Task C2: SYN送信機能
**何をする**: 指定サーバーへSYNパケット送信
```rust
impl TcpConnection {
    fn send_syn(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.local_seq = generate_isn();
        let syn_packet = self.create_syn_packet();
        self.send_tcp_packet(&syn_packet, &[])?;
        self.state = TcpState::SynSent;
        println!("SYN sent: seq={}", self.local_seq);
        Ok(())
    }
}
```

#### Task C3: 状態変更
**何をする**: SYN送信後の状態管理
- CLOSED → SYN-SENT への遷移
- ログ出力での状態確認

#### Task C4: SYNテスト
**何をする**: SYN送信機能のテスト
```rust
#[test]
fn test_syn_packet_creation() {
    let conn = TcpConnection::new(
        u32::from_be_bytes([127, 0, 0, 1]), 
        80
    ).unwrap();
    
    let syn = conn.create_syn_packet();
    let bytes = syn.to_bytes();
    
    // SYNフラグの確認
    assert_eq!(bytes[13] & 0x02, 0x02);
    // その他フィールドの確認
}
```

---

### Phase D: SYN-ACKパケット受信処理

#### Task D1: タイムアウト付き受信
**何をする**: 指定時間内のパケット受信機能
```rust
use std::time::{Duration, Instant};

impl TcpConnection {
    fn receive_packet_timeout(&self, timeout_secs: u64) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let start = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);
        
        loop {
            // ノンブロッキング受信を試行
            match self.try_receive_packet() {
                Ok(data) => return Ok(data),
                Err(_) => {
                    if start.elapsed() > timeout {
                        return Err("Timeout".into());
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }
    }
}
```

#### Task D2: TCPヘッダー解析
**何をする**: 受信パケットからTCPヘッダーを抽出
```rust
impl TcpConnection {
    fn parse_received_packet(&self, data: &[u8]) -> Result<TcpHeader, &'static str> {
        // IPヘッダー長を計算（可変長の可能性）
        if data.len() < 20 {
            return Err("Packet too short");
        }
        
        let ip_header_len = ((data[0] & 0x0F) * 4) as usize;
        if data.len() < ip_header_len + 20 {
            return Err("TCP header incomplete");
        }
        
        // TCPヘッダー部分を抽出
        let tcp_data = &data[ip_header_len..];
        TcpHeader::from_bytes(tcp_data)
    }
}
```

#### Task D3: SYN-ACK検証
**何をする**: 受信パケットがSYN-ACKかチェック
```rust
impl TcpConnection {
    fn is_syn_ack(&self, header: &TcpHeader) -> bool {
        // SYN + ACKフラグの確認
        let flags = header.get_flags(); // 要実装
        let has_syn = (flags & 0x02) != 0;
        let has_ack = (flags & 0x10) != 0;
        
        has_syn && has_ack
    }
    
    fn is_correct_syn_ack(&self, header: &TcpHeader) -> bool {
        self.is_syn_ack(header) && 
        header.get_ack_number() == self.local_seq + 1 &&
        header.get_destination_port() == self.local_port
    }
}
```

#### Task D4: ACK番号チェック
**何をする**: SYN-ACKのACK番号が正しいかチェック
- 期待値: local_seq + 1
- SYNパケットのseq番号に対する正しい応答かチェック

---

### Phase E: ACKパケット送信と接続完了

#### Task E1: ACKパケット構築
**何をする**: SYN-ACK応答のACKパケット作成
```rust
impl TcpConnection {
    fn create_ack_packet(&self, ack_number: u32) -> TcpHeader {
        let mut header = TcpHeader::new(
            self.local_port,
            self.remote_port,
            self.local_seq + 1,    // SYN送信後なので+1
            ack_number,            // 相手のseq + 1
            0x10,                  // ACKフラグ（bit 4）
            8192,                  // ウィンドウサイズ
        );
        
        // チェックサム計算
        header.calculate_checksum(
            u32::from_be_bytes([127, 0, 0, 1]),
            self.remote_ip,
            &[]
        );
        
        header
    }
}
```

#### Task E2: ACK送信
**何をする**: ACKパケットの送信機能
```rust
impl TcpConnection {
    fn send_ack(&mut self, ack_number: u32) -> Result<(), Box<dyn std::error::Error>> {
        let ack_packet = self.create_ack_packet(ack_number);
        self.send_tcp_packet(&ack_packet, &[])?;
        self.remote_seq = ack_number - 1; // 相手のシーケンス番号を記録
        println!("ACK sent: seq={}, ack={}", self.local_seq + 1, ack_number);
        Ok(())
    }
}
```

#### Task E3: 接続確立完了
**何をする**: 状態をESTABLISHEDに変更
```rust
impl TcpConnection {
    fn complete_handshake(&mut self) {
        self.state = TcpState::Established;
        self.local_seq += 1; // SYNの消費
        println!("Connection established!");
    }
}
```

#### Task E4: 接続確認
**何をする**: 接続が正常に確立されたかチェック
```rust
impl TcpConnection {
    fn is_connected(&self) -> bool {
        self.state == TcpState::Established
    }
}
```

---

### Phase F: 統合テストと動作確認

#### Task F1: 完全handshakeテスト
**何をする**: 全フローを統合したテスト
```rust
impl TcpConnection {
    fn connect(&mut self, timeout_secs: u64) -> Result<(), Box<dyn std::error::Error>> {
        // 1. SYN送信
        self.send_syn()?;
        
        // 2. SYN-ACK受信
        let response = self.receive_packet_timeout(timeout_secs)?;
        let syn_ack = self.parse_received_packet(&response)?;
        
        if !self.is_correct_syn_ack(&syn_ack) {
            return Err("Invalid SYN-ACK received".into());
        }
        
        // 3. ACK送信
        let ack_number = syn_ack.get_sequence_number() + 1;
        self.send_ack(ack_number)?;
        
        // 4. 接続完了
        self.complete_handshake();
        
        Ok(())
    }
}
```

#### Task F2: 実サーバーテスト
**何をする**: 実際のWebサーバーとの接続
```rust
#[test]
fn test_real_server_connection() {
    // httpbin.org (IP: 54.92.72.139)への接続テスト
    let remote_ip = u32::from_be_bytes([54, 92, 72, 139]);
    let mut conn = TcpConnection::new(remote_ip, 80).unwrap();
    
    // 3-way handshake実行
    assert!(conn.connect(5).is_ok());
    assert!(conn.is_connected());
}
```

#### Task F3: Wiresharkキャプチャ
**何をする**: パケットキャプチャでの動作確認
```bash
# 実行前にWiresharkでキャプチャ開始
sudo wireshark -i lo -f "tcp port 80" &

# プログラム実行
sudo cargo run --bin step03

# パケット確認ポイント:
# 1. SYNパケット（flags: S）
# 2. SYN-ACKパケット（flags: SA）  
# 3. ACKパケット（flags: A）
# 4. seq/ack番号の正確性
```

#### Task F4: エラーケーステスト
**何をする**: 異常ケースの処理確認
```rust
#[test]
fn test_connection_timeout() {
    // 存在しないサーバーへの接続
    let mut conn = TcpConnection::new(
        u32::from_be_bytes([192, 168, 255, 254]), 
        12345
    ).unwrap();
    
    // タイムアウトすることを確認
    assert!(conn.connect(1).is_err());
}

#[test]
fn test_connection_refused() {
    // ローカルの未使用ポートへの接続
    let mut conn = TcpConnection::new(
        u32::from_be_bytes([127, 0, 0, 1]), 
        65432
    ).unwrap();
    
    // 接続拒否されることを確認
    assert!(conn.connect(2).is_err());
}
```

---

## 🚀 実装開始の手順

### 1. まず始めること
```bash
cd src/step03
touch main.rs tests.rs LEARNING_LOG.md
```

### 2. main.rsの基本構造
```rust
use std::net::Ipv4Addr;
use std::time::{SystemTime, UNIX_EPOCH, Duration, Instant};

// Step2からTcpHeaderを再利用
// use crate::step02::{TcpHeader, calculate_checksum_rfc1071}; // プロジェクト構成次第で調整

// Raw socketの基本機能（Step1から再利用）
mod socket_utils {
    // create_raw_socket, send_packet等
}

#[derive(Debug, Clone, PartialEq)]
pub enum TcpState {
    Closed,
    SynSent,
    Established,
}

#[derive(Debug)]
pub struct TcpConnection {
    // Task B1で実装
}

impl TcpConnection {
    fn new(remote_ip: u32, remote_port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        // Task B1で実装
    }
    
    fn connect(&mut self, timeout_secs: u64) -> Result<(), Box<dyn std::error::Error>> {
        // Task F1で実装（全フローの統合）
    }
}

fn generate_isn() -> u32 {
    // Task B2で実装
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Step 3: 3-way Handshake Implementation");
    
    // デモ実行
    let remote_ip = u32::from_be_bytes([127, 0, 0, 1]); // localhost
    let mut conn = TcpConnection::new(remote_ip, 80)?;
    
    println!("Attempting 3-way handshake with {}:80", 
             Ipv4Addr::from(remote_ip));
    
    match conn.connect(5) {
        Ok(_) => println!("Successfully established connection!"),
        Err(e) => println!("Connection failed: {}", e),
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // 各Phaseのテストをここに実装
}
```

### 3. 段階的実装のコツ
1. **Phase単位で実装**: A→B→C→D→E→Fの順序で進める
2. **テスト駆動**: 各Taskにテストを書いてから実装
3. **ログ活用**: パケット内容や状態変化を詳細にログ出力
4. **既存コード活用**: Step1/2の実装を最大限再利用

### 4. デバッグのポイント
- **パケットキャプチャ**: Wiresharkでの実パケット確認
- **バイト値チェック**: TCPフラグ等のビット操作を16進数で確認
- **タイミング**: パケット送受信のタイミング問題に注意
- **チェックサム**: Step2の経験を活かしてチェックサム検証

---

## 📝 完了チェックリスト

Step3完了の条件：
- [ ] 全6つのPhase（A-F）が完了している
- [ ] 単体テストが全て成功している
- [ ] 実際のサーバーとの3-way handshakeが成功
- [ ] Wiresharkでパケット内容を確認済み
- [ ] エラーケースも適切に処理できる
- [ ] LEARNING_LOG.mdに学習内容を記録済み

**推定所要時間**: 約5時間（休憩含む）

**次のステップ**: Step 4でTCP状態マシンの完全実装に進みます！

## 🔗 参考資料

### RFC関連
- **RFC 9293 Section 3.5**: Establishing a connection（必読）
- **RFC 9293 Section 3.8.1**: Connection establishment example
- **RFC 6528**: Defending Against Sequence Number Attacks（ISN生成）

### 実装パターン
- Step1のRaw socket操作
- Step2のTCPヘッダー処理とチェックサム計算
- 状態管理のパターン（次のStep4で詳しく学習）
