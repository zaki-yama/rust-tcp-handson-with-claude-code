# Step 4: TCP State Machine（TCP状態マシン）

## 目標

TCP状態マシンの完全な実装を通じて、TCPプロトコルの状態管理と遷移ロジックを理解します。

## 学習内容

- TCP状態の種類と意味（11種類の状態）
- 状態遷移のルールとトリガー
- イベント駆動型処理の設計パターン
- 接続確立・データ転送・接続終了の各フェーズでの状態管理

### Step03との関係

Step03とStep04は、異なる目的で設計されています：

| 項目 | Step03: TcpConnection | Step04: TcpStateMachine |
|------|----------------------|------------------------|
| **責務** | TCP接続全体の管理 | 状態遷移のロジックのみ |
| **状態数** | 3状態（Closed, SynSent, Established） | 11状態（RFC 9293完全準拠） |
| **機能** | パケット送受信、接続管理 | 状態遷移の検証と履歴管理 |
| **ネットワーク** | Raw socketを直接操作 | ネットワーク操作なし（純粋なロジック） |
| **目的** | 実用的な3-way handshake実装 | 完全な状態マシンの学習 |

**Step03**: 実際のネットワーク通信を行う簡略化されたTCP実装（3-way handshakeに焦点）
- `TcpConnection`がソケット操作と状態を直接管理
- 接続確立（SYN → SYN-ACK → ACK）の実装
- 実用性重視

**Step04**: 理論的で完全な状態マシンの実装（RFC準拠、学習用）
- `TcpStateMachine`が状態遷移のロジックのみを担当
- RFC 9293の11状態すべてを実装
- 接続終了（FIN処理）やエラーケース（RST）も含む
- 状態遷移の検証と履歴管理
- 理論と設計パターンの理解重視

**将来的な統合**: Step05以降では、Step04で学んだ状態マシンをStep03のような実装に組み込むことで、より堅牢なTCP実装を構築します。

```rust
// 将来的な統合例
pub struct TcpConnection {
    socket_fd: i32,
    state_machine: TcpStateMachine,  // ← 状態管理を委譲
    local_seq: u32,
    remote_seq: u32,
    // ...
}
```

このステップでは、ネットワーク操作から切り離された純粋な状態マシンの実装に集中することで、TCP状態遷移の本質を深く理解できます。

## RFC参照

**RFC 9293 - Section 3.3.2 (State Machine Overview), Figure 5 (TCP Connection State Diagram)**
- TCP状態遷移図の詳細
- 各状態の定義と遷移条件
- イベント処理とアクション

https://www.rfc-editor.org/rfc/rfc9293.html#section-3.3.2

## 実装の流れ

Step04では、以下の6つのフェーズに分けて実装を進めます：

### Phase A: TCP状態の定義（15-20分）
基本的な状態列挙型と構造を定義します。

### Phase B: 状態遷移ロジック（25-30分）
イベントに基づく状態遷移を実装します。

### Phase C: 接続確立時の状態管理（20-25分）
3-way handshake時の状態遷移を実装します。

### Phase D: 接続終了時の状態管理（25-30分）
4-way handshake（FIN処理）時の状態遷移を実装します。

### Phase E: エラーハンドリングと異常系（20-25分）
RSTパケットや不正な状態遷移の処理を実装します。

### Phase F: 統合テストと検証（15-20分）
完全な状態遷移シナリオのテストを実行します。

---

## Phase A: TCP状態の定義

### Task A1: TcpState列挙型の定義
RFC 9293で定義されている11種類のTCP状態を列挙型として定義します。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    Closed,       // 接続なし
    Listen,       // 接続待ち受け中
    SynSent,      // SYN送信済み、SYN-ACK待ち
    SynReceived,  // SYN受信済み、ACK待ち
    Established,  // 接続確立
    FinWait1,     // FIN送信済み、ACK待ち
    FinWait2,     // FIN ACK受信、相手のFIN待ち
    CloseWait,    // 相手のFIN受信、自分のFIN送信待ち
    Closing,      // 両方のFIN送信済み、ACK待ち
    LastAck,      // FIN送信済み、最後のACK待ち
    TimeWait,     // 接続終了後の待機期間
}
```

**実装ヒント**:
- RFC 9293 Figure 5を参照
- `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`を付与
- Step03では3状態のみでしたが、完全な11状態に拡張

### Task A2: TcpEvent列挙型の定義
状態遷移のトリガーとなるイベントを定義します。

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TcpEvent {
    Connect,              // アプリケーションからの接続要求
    Listen,               // アプリケーションからの待ち受け要求
    ReceiveSyn,           // SYNパケット受信
    ReceiveSynAck,        // SYN-ACKパケット受信
    ReceiveAck,           // ACKパケット受信
    ReceiveFin,           // FINパケット受信
    ReceiveRst,           // RSTパケット受信
    Close,                // アプリケーションからの切断要求
    Timeout,              // タイムアウト
}
```

**実装ヒント**:
- イベントは外部（ネットワーク、アプリケーション）から発生
- 各イベントが特定の状態遷移をトリガー

### Task A3: StateMachineの基本構造
状態マシンを管理する構造体を定義します。

```rust
pub struct TcpStateMachine {
    current_state: TcpState,
    // 状態遷移履歴（デバッグ用）
    state_history: Vec<(TcpState, TcpState, TcpEvent)>,
}
```

**実装ヒント**:
- `current_state`で現在の状態を保持
- `state_history`で状態遷移の履歴を記録（デバッグに有用）

---

## Phase B: 状態遷移ロジック

### Task B1: 状態遷移メソッドの実装
イベントに基づいて状態を遷移させるメソッドを実装します。

```rust
impl TcpStateMachine {
    pub fn transition(&mut self, event: TcpEvent) -> Result<TcpState, String> {
        let old_state = self.current_state;
        let new_state = self.next_state(old_state, event.clone())?;

        self.current_state = new_state;
        self.state_history.push((old_state, new_state, event));

        Ok(new_state)
    }

    fn next_state(&self, current: TcpState, event: TcpEvent) -> Result<TcpState, String> {
        // Task B2で実装
    }
}
```

**実装ヒント**:
- 状態遷移の妥当性をチェック
- 不正な遷移はエラーを返す
- 履歴に記録してデバッグを容易に

### Task B2: 状態遷移テーブルの実装
各状態とイベントの組み合わせに対する遷移先を定義します。

```rust
fn next_state(&self, current: TcpState, event: TcpEvent) -> Result<TcpState, String> {
    use TcpEvent::*;
    use TcpState::*;

    match (current, event.clone()) {
        // Closed状態からの遷移
        (Closed, Connect) => Ok(SynSent),
        (Closed, Listen) => Ok(Listen),

        // SynSent状態からの遷移
        (SynSent, ReceiveSynAck) => Ok(Established),
        (SynSent, ReceiveSyn) => Ok(SynReceived),

        // Established状態からの遷移
        (Established, Close) => Ok(FinWait1),
        (Established, ReceiveFin) => Ok(CloseWait),

        // ... 他の遷移を実装

        // 不正な遷移
        _ => Err(format!("Invalid transition: {:?} -> {:?}", current, event)),
    }
}
```

**実装ヒント**:
- RFC 9293 Figure 5の状態遷移図を参照
- すべての有効な遷移パターンをカバー
- 不正な遷移は明確なエラーメッセージを返す

### Task B3: 状態確認メソッド
現在の状態を確認するヘルパーメソッドを実装します。

```rust
impl TcpStateMachine {
    pub fn is_established(&self) -> bool {
        self.current_state == TcpState::Established
    }

    pub fn is_closed(&self) -> bool {
        self.current_state == TcpState::Closed
    }

    pub fn can_send_data(&self) -> bool {
        matches!(self.current_state, TcpState::Established | TcpState::CloseWait)
    }

    pub fn can_receive_data(&self) -> bool {
        matches!(self.current_state, TcpState::Established | TcpState::FinWait1 | TcpState::FinWait2)
    }
}
```

**実装ヒント**:
- よく使う状態チェックをメソッド化
- データ送受信可能かの判定を提供

---

## Phase C: 接続確立時の状態管理

### Task C1: アクティブオープンの実装
クライアント側（接続を開始する側）の状態遷移を実装します。

**シナリオ**: Closed → SynSent → Established

```rust
impl TcpStateMachine {
    pub fn active_open(&mut self) -> Result<(), String> {
        // 1. Connect イベントで Closed → SynSent
        self.transition(TcpEvent::Connect)?;
        Ok(())
    }

    pub fn complete_active_open(&mut self) -> Result<(), String> {
        // 2. SYN-ACK受信で SynSent → Established
        self.transition(TcpEvent::ReceiveSynAck)?;
        Ok(())
    }
}
```

**実装ヒント**:
- Step03の3-way handshakeと対応
- クライアント視点の状態遷移

### Task C2: パッシブオープンの実装
サーバー側（接続を待ち受ける側）の状態遷移を実装します。

**シナリオ**: Closed → Listen → SynReceived → Established

```rust
impl TcpStateMachine {
    pub fn passive_open(&mut self) -> Result<(), String> {
        // 1. Listen イベントで Closed → Listen
        self.transition(TcpEvent::Listen)?;
        Ok(())
    }

    pub fn accept_connection(&mut self) -> Result<(), String> {
        // 2. SYN受信で Listen → SynReceived
        self.transition(TcpEvent::ReceiveSyn)?;

        // 3. ACK受信で SynReceived → Established
        self.transition(TcpEvent::ReceiveAck)?;
        Ok(())
    }
}
```

**実装ヒント**:
- サーバー側の接続確立フロー
- Listenステートの扱い

### Task C3: 同時オープンの処理
両方がSYNを送信するケースの処理を実装します。

**シナリオ**: SynSent → SynReceived → Established

```rust
impl TcpStateMachine {
    pub fn simultaneous_open(&mut self) -> Result<(), String> {
        // SynSent状態でSYN受信 → SynReceived
        self.transition(TcpEvent::ReceiveSyn)?;

        // SYN-ACK受信 → Established
        self.transition(TcpEvent::ReceiveSynAck)?;
        Ok(())
    }
}
```

**実装ヒント**:
- RFC 9293 Section 3.5参照
- 稀なケースだが仕様上サポート必要

---

## Phase D: 接続終了時の状態管理

### Task D1: アクティブクローズの実装
自分から接続を切断する側の状態遷移を実装します。

**シナリオ**: Established → FinWait1 → FinWait2 → TimeWait → Closed

```rust
impl TcpStateMachine {
    pub fn active_close(&mut self) -> Result<(), String> {
        // 1. FIN送信: Established → FinWait1
        self.transition(TcpEvent::Close)?;

        // 2. ACK受信: FinWait1 → FinWait2
        self.transition(TcpEvent::ReceiveAck)?;

        // 3. FIN受信: FinWait2 → TimeWait
        self.transition(TcpEvent::ReceiveFin)?;

        // 4. タイムアウト: TimeWait → Closed
        // (2MSL後)
        Ok(())
    }
}
```

**実装ヒント**:
- 4-way handshakeの実装
- TimeWait状態の重要性（2MSL待機）

### Task D2: パッシブクローズの実装
相手から切断された場合の状態遷移を実装します。

**シナリオ**: Established → CloseWait → LastAck → Closed

```rust
impl TcpStateMachine {
    pub fn passive_close(&mut self) -> Result<(), String> {
        // 1. FIN受信: Established → CloseWait
        self.transition(TcpEvent::ReceiveFin)?;

        // 2. FIN送信: CloseWait → LastAck
        self.transition(TcpEvent::Close)?;

        // 3. ACK受信: LastAck → Closed
        self.transition(TcpEvent::ReceiveAck)?;
        Ok(())
    }
}
```

**実装ヒント**:
- 受動的な切断処理
- CloseWait状態でアプリケーションの切断を待つ

### Task D3: 同時クローズの処理
両方が同時にFINを送信するケースを実装します。

**シナリオ**: Established → FinWait1 → Closing → TimeWait → Closed

```rust
impl TcpStateMachine {
    pub fn simultaneous_close(&mut self) -> Result<(), String> {
        // 1. FIN送信: Established → FinWait1
        self.transition(TcpEvent::Close)?;

        // 2. FIN受信: FinWait1 → Closing
        self.transition(TcpEvent::ReceiveFin)?;

        // 3. ACK受信: Closing → TimeWait
        self.transition(TcpEvent::ReceiveAck)?;
        Ok(())
    }
}
```

**実装ヒント**:
- 稀なケースだが仕様上必要
- Closing状態の扱い

---

## Phase E: エラーハンドリングと異常系

### Task E1: RSTパケット処理
リセットパケット受信時の処理を実装します。

```rust
impl TcpStateMachine {
    pub fn handle_reset(&mut self) -> Result<(), String> {
        // 任意の状態からClosedへ即座に遷移
        if self.current_state != TcpState::Closed {
            self.current_state = TcpState::Closed;
            self.state_history.push((
                self.current_state,
                TcpState::Closed,
                TcpEvent::ReceiveRst,
            ));
        }
        Ok(())
    }
}
```

**実装ヒント**:
- RSTは即座に接続をリセット
- どの状態からでもClosedに遷移可能

### Task E2: タイムアウト処理
タイムアウト時の状態遷移を実装します。

```rust
impl TcpStateMachine {
    pub fn handle_timeout(&mut self) -> Result<(), String> {
        match self.current_state {
            TcpState::SynSent | TcpState::SynReceived => {
                // 接続確立タイムアウト → Closed
                self.transition(TcpEvent::Timeout)?;
                self.current_state = TcpState::Closed;
            }
            TcpState::TimeWait => {
                // 2MSL経過 → Closed
                self.current_state = TcpState::Closed;
            }
            _ => {
                return Err(format!("Timeout not applicable in {:?}", self.current_state));
            }
        }
        Ok(())
    }
}
```

**実装ヒント**:
- 状態ごとにタイムアウト処理が異なる
- TimeWaitの2MSL後にClosed

### Task E3: 不正遷移の検出とログ
不正な状態遷移を検出して記録します。

```rust
impl TcpStateMachine {
    pub fn get_state_history(&self) -> &[(TcpState, TcpState, TcpEvent)] {
        &self.state_history
    }

    pub fn print_state_diagram(&self) {
        println!("State Transition History:");
        for (i, (from, to, event)) in self.state_history.iter().enumerate() {
            println!("  {}. {:?} --[{:?}]--> {:?}", i + 1, from, event, to);
        }
    }
}
```

**実装ヒント**:
- デバッグ用の状態遷移履歴表示
- 問題解析に役立つ

---

## Phase F: 統合テストと検証

### Task F1: 完全な接続確立・終了テスト
接続の開始から終了までの完全なフローをテストします。

```rust
#[test]
fn test_full_connection_lifecycle() {
    let mut sm = TcpStateMachine::new();

    // アクティブオープン
    sm.active_open().unwrap();
    assert_eq!(sm.current_state(), TcpState::SynSent);

    sm.complete_active_open().unwrap();
    assert_eq!(sm.current_state(), TcpState::Established);

    // アクティブクローズ
    sm.active_close().unwrap();
    // ... 検証
}
```

### Task F2: エラーケースの検証
異常系のテストを実装します。

```rust
#[test]
fn test_invalid_transitions() {
    let mut sm = TcpStateMachine::new();

    // Closed状態でFIN受信は不正
    let result = sm.transition(TcpEvent::ReceiveFin);
    assert!(result.is_err());
}
```

### Task F3: RFC準拠の確認
RFC 9293の状態遷移図と実装が一致することを確認します。

---

## 完成チェックリスト

- [ ] Phase A: 11種類のTCP状態を定義
- [ ] Phase B: 状態遷移ロジックを実装
- [ ] Phase C: 接続確立時の状態管理を実装
- [ ] Phase D: 接続終了時の状態管理を実装
- [ ] Phase E: エラーハンドリングを実装
- [ ] Phase F: すべてのテストがパス
- [ ] RFC 9293 Figure 5との対応を確認
- [ ] 状態遷移履歴のデバッグ出力を確認

## 参考リソース

- [RFC 9293 - Transmission Control Protocol](https://www.rfc-editor.org/rfc/rfc9293.html)
- RFC 9293 Section 3.3.2: State Machine Overview
- RFC 9293 Figure 5: TCP Connection State Diagram

## 次のステップ

Step04完了後は、Step05でデータ送受信とシーケンス番号管理を実装します。
