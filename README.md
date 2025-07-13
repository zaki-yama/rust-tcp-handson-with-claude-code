# RustでTCPを自作しながら学ぶハンズオン

このプロジェクトは、RustでTCP/IPプロトコルを段階的に実装しながら、ネットワークプログラミングの基礎を学ぶハンズオン形式の学習コンテンツです。

## 📚 学習目標

- TCP/IPプロトコルの仕組みを実装を通じて理解する
- RFCドキュメントを読みながら、仕様に基づいた実装を行う
- ネットワークプログラミングの基礎知識を身につける
- 段階的に機能を追加し、完全なTCPスタックを構築する

## 🎯 コース概要

### Phase 1: 基礎編
1. **Step 1**: 基本的なソケット通信
2. **Step 2**: TCPヘッダーの解析
3. **Step 3**: 3-way handshake の実装

### Phase 2: 接続管理編
4. **Step 4**: 接続状態管理（TCP State Machine）
5. **Step 5**: データ送受信とシーケンス番号
6. **Step 6**: ACK処理とウィンドウサイズ

### Phase 3: 信頼性確保編
7. **Step 7**: タイムアウトと再送制御
8. **Step 8**: 順序制御と重複検出
9. **Step 9**: フロー制御の実装

### Phase 4: 高度な制御編
10. **Step 10**: 輻輳制御（Slow Start、Congestion Avoidance）
11. **Step 11**: 高速再送とFast Recovery
12. **Step 12**: 接続終了処理（4-way handshake）

## 📖 RFC対応表

| Step | 機能 | 主要RFC | 章/セクション |
|------|------|---------|---------------|
| 1-3 | 基本通信・Handshake | RFC 793 | Section 3.2, 3.4 |
| 4 | 状態管理 | RFC 793 | Section 3.2, Figure 6 |
| 5-6 | データ転送・ACK | RFC 793 | Section 3.3, 3.7 |
| 7-8 | 再送・順序制御 | RFC 793 | Section 3.7 |
| 9 | フロー制御 | RFC 793 | Section 3.7 |
| 10-11 | 輻輳制御 | RFC 5681 | Section 3, 4 |
| 12 | 接続終了 | RFC 793 | Section 3.5 |

## 🛠️ 開発環境

### 必要な環境
- Rust 1.70+
- Linux/macOS（raw socketの利用のため）
- `sudo`権限（raw socketアクセス用）

### セットアップ
```bash
# プロジェクトのクローン
git clone <repository-url>
cd rust-tcp

# 依存関係のインストール
cargo build

# テストの実行
cargo test

# 特定のステップの実行例
sudo cargo run --bin step01
```

## 📝 各ステップの構成

各ステップは以下の構成になっています：

```
src/
├── step01/           # Step 1: 基本的なソケット通信
│   ├── main.rs      # メイン実装
│   ├── README.md    # ステップ詳細説明
│   └── tests.rs     # テストコード
├── step02/           # Step 2: TCPヘッダー解析
│   ├── main.rs
│   ├── tcp_header.rs
│   ├── README.md
│   └── tests.rs
├── ...
├── common/           # 共通ライブラリ
│   ├── packet.rs    # パケット操作
│   ├── socket.rs    # ソケット操作
│   └── utils.rs     # ユーティリティ
└── lib.rs
```

## 🧪 テストと動作確認

各ステップには以下のテストが含まれています：

1. **単体テスト**: 個別機能のテスト
2. **統合テスト**: ステップ間の連携テスト
3. **実環境テスト**: 実際のネットワーク通信テスト

### テスト実行方法
```bash
# 全てのテストを実行
cargo test

# 特定のステップのテストを実行
cargo test step01

# 実環境テスト（要sudo権限）
sudo cargo test --test integration_tests
```

## 📋 学習の進め方

### 推奨学習フロー

1. **RFC読解**: 各ステップ開始前に対応するRFCセクションを読む
2. **コード実装**: ステップガイドに従って実装を進める
3. **テスト実行**: 実装後にテストを実行して動作確認
4. **動作検証**: Wiresharkなどでパケットキャプチャして検証
5. **振り返り**: 実装とRFCの対応を確認

### デバッグツール

- **Wireshark**: パケットキャプチャと解析
- **tcpdump**: コマンドラインでのパケットキャプチャ
- **netstat**: 接続状態の確認
- **ss**: ソケット状態の確認

## 📚 参考資料

### 主要RFC
- [RFC 793](https://tools.ietf.org/html/rfc793) - Transmission Control Protocol
- [RFC 5681](https://tools.ietf.org/html/rfc5681) - TCP Congestion Control
- [RFC 6298](https://tools.ietf.org/html/rfc6298) - Computing TCP's Retransmission Timer

### 推奨書籍
- 「マスタリングTCP/IP」
- 「詳解TCP/IP」
- 「The Rust Programming Language」

## 🤝 コントリビューション

バグ報告や改善提案は Issues でお願いします。

## 📄 ライセンス

MIT License

---

## 次のステップ

Step 1から開始して、段階的にTCPスタックを構築していきましょう！

```bash
cd src/step01
cat README.md  # Step 1の詳細な説明を確認
```