# Step03 学習ログ: 3-way handshake実装

## 学習目標
TCP接続確立の基本プロセス（3-way handshake）を実装し、実際のサーバーとの接続を確立する

## Phase A: 3-way handshakeの理解と設計

### Task A1: RFC 9293 Section 3.5の理解

#### 3-way handshakeのフロー
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

#### 各パケットの意味
- **SYN**: 接続開始要求。クライアントの初期シーケンス番号(x)を通知
- **SYN-ACK**: 接続受諾 + サーバーの初期シーケンス番号(y) + クライアントのSYNに対するACK(x+1)
- **ACK**: 接続確立完了。サーバーのSYNに対するACK(y+1)

### Task A2: シーケンス番号とACK番号の計算ルール

| パケット | seq番号 | ack番号 | 説明 |
|----------|---------|---------|------|
| SYN | ISN_client (x) | 0 | クライアントの初期シーケンス |
| SYN-ACK | ISN_server (y) | x+1 | サーバーの初期seq + クライアントseqをACK |
| ACK | x+1 | y+1 | 両方のISNをインクリメントしてACK |

#### 重要なポイント
- SYNフラグ自体が1バイト分のデータとして扱われる（seq番号を+1する）
- ACK番号は「次に受信を期待するシーケンス番号」
- ISN（Initial Sequence Number）は安全性のためランダムに生成

### Task A3: 接続状態の理解

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TcpState {
    Closed,      // 初期状態、接続なし
    SynSent,     // SYN送信済み、SYN-ACK待ち
    Established, // 接続確立完了
}
```

#### 状態遷移
- CLOSED → SYN-SENT: SYNパケット送信時
- SYN-SENT → ESTABLISHED: 正しいSYN-ACK受信後、ACK送信完了時

## 次のステップ
Phase B: 基本構造の実装（TcpConnection構造体、ISN生成など）

---
*学習開始時刻: `date`*