# Step 2: TCPヘッダーの解析

## 🎯 このステップの最終目標
TCPヘッダーの構造を完全に理解し、パース・生成・検証機能を実装する

## 📋 実装タスク一覧（推奨順序）

### Phase A: 基本構造の理解と準備 (30分)
- [x] **Task A1**: RFC 9293 Section 3.1を読んでTCPヘッダー構造を理解 (※2022年更新版)
- [x] **Task A2**: TCPヘッダーの各フィールドサイズと役割を整理
- [x] **Task A3**: 基本的なプロジェクト構成を作成

### Phase B: TCPヘッダー構造体の実装 (45分)
- [x] **Task B1**: `TcpHeader`構造体を定義
- [x] **Task B2**: コンストラクタ`new()`メソッドを実装
- [x] **Task B3**: バイト配列変換`to_bytes()`メソッドを実装
- [x] **Task B4**: 基本的なテストでヘッダー作成を確認

### Phase C: パース機能の実装 (45分)
- [ ] **Task C1**: バイト配列からのパース`parse()`メソッドを実装
- [ ] **Task C2**: エンディアン変換処理を正しく実装
- [ ] **Task C3**: パース結果の検証テストを作成
- [ ] **Task C4**: 既知のTCPパケットでパーステストを実行

### Phase D: チェックサム計算の実装 (60分)
- [ ] **Task D1**: TCP疑似ヘッダーの構造を理解
- [ ] **Task D2**: 疑似ヘッダー生成機能を実装
- [ ] **Task D3**: TCPチェックサム計算機能を実装（RFC 1071準拠）
- [ ] **Task D4**: チェックサム検証機能を実装

### Phase E: 統合テストと動作確認 (30分)
- [ ] **Task E1**: 全機能を統合したテストを作成
- [ ] **Task E2**: 実際のTCPパケットでの動作確認
- [ ] **Task E3**: エラーケースのテストを追加

---

## 📖 各Phaseの詳細ガイド

### Phase A: 基本構造の理解と準備

#### Task A1: RFC理解
**何をする**: RFC 9293 Section 3.1のTCPヘッダー仕様を読む (※RFC 793の2022年更新版)
```
TCPヘッダーの構造（20バイト固定）:
0                   1                   2                   3
0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|          Source Port          |       Destination Port        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        Sequence Number                        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    Acknowledgment Number                      |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|  Data |       |C|E|U|A|P|R|S|F|                               |
| Offset| Rsrvd |W|C|R|C|S|S|Y|I|            Window             |
|       |       |R|E|G|K|H|T|N|N|                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|           Checksum            |         Urgent Pointer        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                           [Options]                           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                                                               :
:                             Data                              :
:                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

        Note that one tick mark represents one bit position.
```

#### Task A2: フィールド整理
**何をする**: 各フィールドのサイズと役割を表で整理
| フィールド | サイズ | 説明 |
|------------|--------|------|
| Source Port | 16bit | 送信元ポート番号 |
| Destination Port | 16bit | 宛先ポート番号 |
| Sequence Number | 32bit | シーケンス番号 |
| Acknowledgment Number | 32bit | 確認応答番号 |
| Data Offset | 4bit | TCPヘッダー長（32bit word単位） |
| Reserved | 6bit | 予約フィールド（0） |
| Flags | 6bit | URG, ACK, PSH, RST, SYN, FIN |
| Window | 16bit | 受信ウィンドウサイズ |
| Checksum | 16bit | チェックサム |
| Urgent Pointer | 16bit | 緊急ポインター |

#### Task A3: プロジェクト構成作成
**何をする**: 必要なファイルとディレクトリを作成
```bash
src/step02/
├── main.rs        # メイン実装（今回作成）
├── README.md      # このファイル
└── tests.rs       # テストコード
```

---

### Phase B: TCPヘッダー構造体の実装

#### Task B1: 構造体定義
**何をする**: `TcpHeader`構造体を定義
**実装内容**:
```rust
#[repr(C, packed)]
struct TcpHeader {
    source_port: u16,
    destination_port: u16,
    sequence_number: u32,
    acknowledgment_number: u32,
    data_offset_and_flags: u16,  // Data Offset(4bit) + Reserved(6bit) + Flags(6bit)
    window_size: u16,
    checksum: u16,
    urgent_pointer: u16,
}
```

**ポイント**:
- `#[repr(C, packed)]`でC言語互換の配置
- ネットワークバイトオーダーで格納

#### Task B2: コンストラクタ実装
**何をする**: `new()`メソッドでヘッダー初期化
**実装内容**:
```rust
impl TcpHeader {
    fn new(src_port: u16, dst_port: u16, seq: u32, ack: u32, flags: u8, window: u16) -> Self {
        // 実装ヒント:
        // - ポート番号: to_be()でネットワークバイトオーダー変換
        // - Data Offset: 5 (20バイト ÷ 4)
        // - フラグ: 下位6bitに設定
    }
}
```

#### Task B3: バイト配列変換
**何をする**: `to_bytes()`でバイト配列に変換
**実装ヒント**:
- Step1の`IpHeader::to_bytes()`を参考
- ネットワークバイトオーダーを保持

#### Task B4: 基本テスト
**何をする**: ヘッダー作成の基本テスト
```rust
#[test]
fn test_tcp_header_creation() {
    let header = TcpHeader::new(80, 12345, 1000, 0, 0x02, 8192); // SYNフラグ
    let bytes = header.to_bytes();
    assert_eq!(bytes.len(), 20);
    // さらなる検証...
}
```

---

### Phase C: パース機能の実装

#### Task C1: パース実装
**何をする**: `parse()`メソッドでバイト配列からヘッダー作成
**実装内容**:
```rust
impl TcpHeader {
    fn parse(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 20 {
            return Err("Data too short for TCP header");
        }
        
        // バイト配列から各フィールドを抽出
        // ヒント: u16::from_be_bytes(), u32::from_be_bytes()を使用
    }
}
```

#### Task C2: エンディアン処理
**何をする**: パース時の正しいエンディアン変換
**ポイント**:
- `from_be_bytes()`でネットワーク→ホストバイトオーダー変換
- 各フィールドのサイズに注意

#### Task C3: パース検証テスト
**何をする**: パース結果の正確性を検証
```rust
#[test]
fn test_parse_known_packet() {
    let tcp_bytes = [
        0x00, 0x50, 0x30, 0x39,  // src=80, dst=12345
        0x00, 0x00, 0x03, 0xe8,  // seq=1000
        // ... 残りのバイト
    ];
    let header = TcpHeader::parse(&tcp_bytes).unwrap();
    assert_eq!(header.source_port, 80);
    assert_eq!(header.destination_port, 12345);
}
```

#### Task C4: 実パケットテスト
**何をする**: Wiresharkでキャプチャした実際のTCPパケットでテスト

---

### Phase D: チェックサム計算の実装

#### Task D1: 疑似ヘッダー理解
**何をする**: TCP疑似ヘッダーの構造を理解
```
TCP疑似ヘッダー (12バイト):
+---------+---------+---------+---------+
|           Source Address              |
+---------+---------+---------+---------+
|         Destination Address           |
+---------+---------+---------+---------+
|  zero   |  PTCL   |    TCP Length     |
+---------+---------+---------+---------+
```

#### Task D2: 疑似ヘッダー生成
**何をする**: 疑似ヘッダー生成機能を実装
```rust
fn create_pseudo_header(src_ip: u32, dst_ip: u32, tcp_length: u16) -> Vec<u8> {
    // 12バイトの疑似ヘッダーを作成
    // プロトコル番号: 6 (TCP)
}
```

#### Task D3: チェックサム計算
**何をする**: RFC 1071準拠のチェックサム計算
```rust
impl TcpHeader {
    fn calculate_checksum(&mut self, src_ip: u32, dst_ip: u32, tcp_data: &[u8]) {
        // 1. チェックサムフィールドを0にクリア
        // 2. 疑似ヘッダー + TCPヘッダー + データでチェックサム計算
        // 3. 結果をチェックサムフィールドに設定
    }
}
```

#### Task D4: チェックサム検証
**何をする**: 受信パケットのチェックサム検証
```rust
impl TcpHeader {
    fn verify_checksum(&self, src_ip: u32, dst_ip: u32, tcp_data: &[u8]) -> bool {
        // チェックサム含めて計算し、結果が0xFFFFかチェック
    }
}
```

---

### Phase E: 統合テストと動作確認

#### Task E1: 統合テスト
**何をする**: 全機能を組み合わせたテスト
```rust
#[test]
fn test_full_tcp_header_cycle() {
    // 作成 → バイト変換 → パース → チェックサム検証
}
```

#### Task E2: 実パケット確認
**何をする**: tcpdumpやWiresharkでの動作確認

#### Task E3: エラーケーステスト
**何をする**: 不正データでのエラーハンドリング確認

---

## 🚀 実装開始の手順

### 1. まず始めること
```bash
cd src/step02
touch main.rs tests.rs
```

### 2. main.rsの基本構造
```rust
use std::net::Ipv4Addr;

const TCP_HEADER_SIZE: usize = 20;

#[repr(C, packed)]
struct TcpHeader {
    // Task B1で実装
}

impl TcpHeader {
    fn new(/* パラメータ */) -> Self {
        // Task B2で実装
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        // Task B3で実装
    }
    
    fn parse(data: &[u8]) -> Result<Self, &'static str> {
        // Task C1で実装
    }
}

fn main() {
    println!("Step 2: TCP Header Analysis");
    
    // テスト用のサンプル実行
    let header = TcpHeader::new(80, 12345, 1000, 0, 0x02, 8192);
    let bytes = header.to_bytes();
    println!("TCP Header bytes: {:?}", bytes);
    
    // パーステスト
    if let Ok(parsed) = TcpHeader::parse(&bytes) {
        println!("Parse successful!");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Task B4, C3, D4, E1のテストをここに実装
}
```

### 3. 順次実装のコツ
1. **一つずつ確認**: 各Taskが完了したら必ずテストする
2. **段階的デバッグ**: `println!`でバイト値を確認
3. **RFC参照**: 分からない時はRFC 9293を再読 (※最新標準)
4. **既存コード活用**: Step1のコードパターンを参考にする

---

## 📝 完了チェックリスト

Step2完了の条件：
- [ ] すべてのTaskが完了している
- [ ] テストが全て成功している  
- [ ] 実際のTCPパケットがパースできる
- [ ] チェックサム計算が正確
- [ ] コードが整理されている

**推定所要時間**: 約3.5時間（休憩含む）

これでStep1のような迷いなく、段階的に実装を進められるはずです！
