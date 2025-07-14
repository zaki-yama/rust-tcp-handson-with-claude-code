# Step 1: 基本的なソケット通信

## 🎯 このステップで実現すること

**最終的な動作**: 手作りIPパケットをlocalhost間で送受信し、パケット内容を解析・表示する

### 具体的な動作例
```bash
$ sudo cargo run --bin step01
[INFO] Raw socket created successfully (fd: 3)
[INFO] Sending packet from 127.0.0.1 to 127.0.0.1
[INFO] Sent 38 bytes to 127.0.0.1
[INFO] Received 38 bytes
[INFO] === IP Header Analysis ===
[INFO] Version: 4, Header Length: 20 bytes
[INFO] Protocol: 6 (TCP)
[INFO] Source: 127.0.0.1
[INFO] Destination: 127.0.0.1
```

## 🛠️ 実装が必要な機能

### 1. IpHeader構造体 ✅ 完了
```rust
#[repr(C, packed)]
struct IpHeader {
    version_ihl: u8,      // Version(4bit) + IHL(4bit)
    tos: u8,              // Type of Service
    length: u16,          // Total Length
    id: u16,              // Identification
    flags_fragment: u16,  // Flags + Fragment Offset
    ttl: u8,              // Time to Live
    protocol: u8,         // Protocol (TCP=6)
    checksum: u16,        // Header Checksum
    source: u32,          // Source IP Address
    destination: u32,     // Destination IP Address
}
```

### 2. create_raw_socket() 関数 🔄 実装中
**目的**: TCP用のraw socketを作成し、カスタムIPヘッダーを有効化
```rust
fn create_raw_socket() -> Result<i32, Box<dyn std::error::Error>> {
    // 1. libc::socket(AF_INET, SOCK_RAW, IPPROTO_TCP)でソケット作成
    // 2. IP_HDRINCLオプションを設定してカスタムヘッダーを有効化
    // 3. エラーハンドリング
}
```

### 3. IpHeader::new() メソッド ⭐ 次に実装
**目的**: 送信用IPヘッダーを初期化
```rust
impl IpHeader {
    fn new(source: Ipv4Addr, dest: Ipv4Addr, data_len: u16) -> Self {
        // 各フィールドを適切な値で初期化
        // ネットワークバイトオーダーに変換
    }
}
```

### 4. IpHeader::calculate_checksum() メソッド ⭐ 次に実装
**目的**: IPヘッダーチェックサムを計算
```rust
impl IpHeader {
    fn calculate_checksum(&mut self) {
        // RFC 791のチェックサムアルゴリズム実装
        // 16ビット単位での加算とキャリー処理
    }
}
```

### 5. send_packet() 関数 ⭐ 次に実装
**目的**: IPパケットを作成して送信
```rust
fn send_packet(socket_fd: i32, source: Ipv4Addr, dest: Ipv4Addr, data: &[u8]) -> Result<(), Error> {
    // 1. IPヘッダー作成
    // 2. チェックサム計算
    // 3. データと結合
    // 4. sendto()で送信
}
```

### 6. receive_packet() 関数 ⭐ 次に実装
**目的**: パケットを受信（ノンブロッキング）
```rust
fn receive_packet(socket_fd: i32) -> Result<Vec<u8>, Error> {
    // recv()でデータ受信
    // MSG_DONTWAITでノンブロッキング
}
```

### 7. parse_ip_header() 関数 ⭐ 次に実装
**目的**: 受信パケットのIPヘッダーを解析・表示
```rust
fn parse_ip_header(data: &[u8]) -> Result<(), Error> {
    // バイト配列からフィールド抽出
    // 各フィールドをログ出力
}
```

### 8. main() 関数 ⭐ 次に実装
**目的**: 全体の処理フローを制御
```rust
fn main() -> Result<(), Error> {
    // 1. ログ初期化
    // 2. raw socket作成
    // 3. サンプルデータ送信
    // 4. パケット受信・解析
    // 5. ソケットクローズ
}
```

## 📚 学習内容

### Raw Socketとは

Raw socketは、OSのネットワークスタックを部分的にバイパスして、直接パケットを送受信できる仕組みです。通常のTCPソケットとは異なり、プロトコルヘッダーを手動で構築する必要があります。

### IPパケットの基本構造

```
IPv4 Header (20 bytes minimum)
+-------+-------+-------+-------+
|Version|  IHL  |   ToS |   Length   |
+-------+-------+-------+-------+
|    Identification   |Flags|Fragment|
+-------+-------+-------+-------+
|  TTL  |Protocol|  Header Checksum |
+-------+-------+-------+-------+
|          Source Address          |
+-------+-------+-------+-------+
|        Destination Address       |
+-------+-------+-------+-------+
|            Options...            |
+-------+-------+-------+-------+
```

## 🛠️ 実装内容

### 1. Raw Socketの作成
```rust
use std::net::Ipv4Addr;
use std::os::unix::io::AsRawFd;
use libc::{socket, AF_INET, SOCK_RAW, IPPROTO_TCP};

// Raw socketを作成
let socket_fd = unsafe {
    socket(AF_INET, SOCK_RAW, IPPROTO_TCP)
};
```

### 2. IPヘッダーの構築
- バージョン、ヘッダー長、サービスタイプ
- パケット長、識別子、フラグ
- TTL、プロトコル、チェックサム
- 送信元・宛先IPアドレス

### 3. パケット送信
- sockaddr_in構造体の設定
- sendto()システムコールでパケット送信

### 4. パケット受信
- recv()システムコールでパケット受信
- 受信データの解析とログ出力

## 📖 RFC参照

**RFC 791 - Section 3.1**: Internet Header Format
- IPヘッダーの詳細構造
- 各フィールドの意味と使用方法
- チェックサム計算アルゴリズム

**RFC 9293 - Section 2**: TCP Overview
- TCP/IPアーキテクチャの基本概念
- TCPとIPの関係

## 🔧 必要な準備

### 権限要件
Raw socketを使用するためには管理者権限が必要です：
```bash
sudo cargo run --bin step01
```

### 依存関係
- `libc`: システムコール用
- `log`: ログ出力用
- `env_logger`: ログ設定用

## 💻 実行方法

```bash
# Step 1の実行
sudo cargo run --bin step01

# ログレベルを指定して実行
sudo RUST_LOG=debug cargo run --bin step01

# 別ターミナルでパケットキャプチャ
sudo tcpdump -i lo -v host 127.0.0.1
```

## ✅ 動作確認

### 1. コンパイル確認
```bash
cargo check --bin step01
```

### 2. 基本動作確認
- プログラムが正常に起動するか
- Raw socketが作成できるか
- パケット送信が成功するか

### 3. パケットキャプチャ確認
```bash
# tcpdumpでパケット確認
sudo tcpdump -i lo -X host 127.0.0.1

# Wiresharkでの確認
# 1. Wiresharkを起動
# 2. Loopbackインターフェースを選択
# 3. フィルター: ip.src == 127.0.0.1
```

### 4. ログ出力確認
- 送信パケットの詳細情報
- IPヘッダーの各フィールド値
- エラーハンドリングの動作

## 🚨 よくある問題

### 権限エラー
```
Permission denied (os error 13)
```
→ `sudo`で実行してください

### ソケット作成エラー
```
Protocol not supported (os error 93)
```
→ プラットフォーム固有の問題。Linuxで実行してください

### アドレス使用エラー
```
Address already in use (os error 98)
```
→ しばらく待ってから再実行してください

## 📝 学習のポイント

1. **システムコールの理解**: Rustから直接Cのシステムコールを呼ぶ方法
2. **メモリレイアウト**: パケットデータのバイト配列での表現
3. **エンディアン**: ネットワークバイトオーダー（ビッグエンディアン）の重要性
4. **エラーハンドリング**: システムレベルでのエラー処理

## ✅ 完了チェックリスト

Step 1完了の確認項目：

### 実装チェック
- [ ] 全8つの関数・メソッドが実装済み
- [ ] `cargo check --bin step01`でコンパイルエラーなし
- [ ] `sudo cargo run --bin step01`で実行できる

### 動作チェック
- [ ] "Raw socket created successfully"のログ出力
- [ ] "Sent X bytes to 127.0.0.1"のログ出力
- [ ] IPヘッダー解析結果の表示
- [ ] tcpdumpでパケットキャプチャ確認

### 理解チェック
- [ ] IPヘッダーの各フィールドの役割を説明できる
- [ ] ネットワークバイトオーダーの必要性を理解
- [ ] チェックサム計算アルゴリズムを理解

**全てクリアしたら**: Step 2でTCPヘッダーの詳細解析に進みます！