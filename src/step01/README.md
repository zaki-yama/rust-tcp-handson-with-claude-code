# Step 1: 基本的なソケット通信

## 🎯 学習目標

- Raw socketの基本的な使用方法を理解する
- IPパケットの構造を学ぶ
- 低レベルネットワークプログラミングの基礎を身につける
- パケットの送受信とログ出力を実装する

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

**RFC 793 - Section 2.1**: 基本概念
- IPプロトコルの役割
- TCPとIPの関係
- パケット構造の概要

**RFC 791 - Internet Protocol**: IP仕様
- IPヘッダーフォーマット
- アドレッシング
- フラグメンテーション

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

## 🔄 次のステップへ

Step 1が完了したら、以下を確認してStep 2に進みましょう：

- [ ] Raw socketでパケット送受信ができる
- [ ] IPヘッダーの基本構造を理解している
- [ ] パケットキャプチャツールでパケットを確認できる
- [ ] ログ出力で実行状況を追跡できる

**次回**: Step 2でTCPヘッダーの詳細な解析を行います！