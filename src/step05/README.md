# Step 5: データ送受信とシーケンス番号管理

## 目標

TCPの基本的なデータ転送機能を実装し、シーケンス番号による順序制御とバッファ管理を理解します。

## 学習内容

- **シーケンス番号の管理**: ISN生成、シーケンス番号の演算とラップアラウンド処理
- **セグメント化とデータ分割**: MSSに基づくデータ分割、効率的なセグメント生成
- **送信・受信バッファ**: データの一時保管、順序制御、効率的なメモリ管理

### Step04との関係

Step04では純粋な状態マシンを学習しました。Step05では、その状態マシンを実際のデータ転送に統合します。

| 項目 | Step04: TcpStateMachine | Step05: データ転送 |
|------|------------------------|-------------------|
| **責務** | 状態遷移のロジック | データ送受信の管理 |
| **主要機能** | 11状態の遷移管理 | バッファ、シーケンス番号、セグメント化 |
| **ネットワーク** | なし（純粋ロジック） | データ送受信を実装 |
| **目的** | 状態管理の学習 | 実用的なデータ転送の実装 |

**統合イメージ**:
```rust
pub struct TcpConnection {
    state_machine: TcpStateMachine,  // Step04で学んだ状態管理
    send_buffer: SendBuffer,         // Step05: 送信バッファ
    recv_buffer: ReceiveBuffer,      // Step05: 受信バッファ
    seq_num: SequenceNumber,         // Step05: シーケンス番号
    // ...
}
```

## RFC参照

**RFC 9293 - Section 3.3 (Sequence Numbers)**
- シーケンス番号の役割と範囲
- ISN（Initial Sequence Number）の生成
- シーケンス番号の比較演算

**RFC 9293 - Section 3.7 (Data Communication)**
- データの送受信プロセス
- バッファ管理
- セグメント化の規則

https://www.rfc-editor.org/rfc/rfc9293.html#section-3.3
https://www.rfc-editor.org/rfc/rfc9293.html#section-3.7

## 実装の流れ

Step05では、以下の6つのフェーズに分けて実装を進めます：

### Phase A: シーケンス番号の定義（20-25分）
基本的なシーケンス番号の構造と演算を定義します。

### Phase B: 送信バッファの実装（25-30分）
送信待ちデータを管理するバッファを実装します。

### Phase C: 受信バッファの実装（25-30分）
受信データを順序通りに並べるバッファを実装します。

### Phase D: セグメント化ロジック（20-25分）
データをMSSサイズのセグメントに分割します。

### Phase E: データ送受信統合（25-30分）
バッファとシーケンス番号を使った送受信処理を実装します。

### Phase F: 統合テストと検証（15-20分）
完全なデータ転送シナリオのテストを実行します。

---

## Phase A: シーケンス番号の定義

### Task A1: SequenceNumber構造体の定義

32ビットのシーケンス番号をラップする構造体を定義します。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequenceNumber(u32);

impl SequenceNumber {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}
```

**実装ヒント**:
- RFC 9293 Section 3.3参照
- シーケンス番号は32ビット符号なし整数
- ラップアラウンド（0xFFFFFFFF → 0x00000000）を考慮した設計が必要

### Task A2: シーケンス番号の演算

シーケンス番号の加算とラップアラウンド処理を実装します。

```rust
impl SequenceNumber {
    pub fn wrapping_add(&self, n: u32) -> Self {
        Self(self.0.wrapping_add(n))
    }

    pub fn wrapping_sub(&self, other: Self) -> u32 {
        self.0.wrapping_sub(other.0)
    }
}
```

**実装ヒント**:
- `wrapping_add` はオーバーフローを自動処理
- 差分計算でデータサイズを求める

### Task A3: シーケンス番号の比較

RFC 793のシーケンス番号比較アルゴリズムを実装します。

```rust
impl SequenceNumber {
    pub fn compare(&self, other: Self) -> std::cmp::Ordering {
        // RFC 793のシーケンス番号比較
        // SEG.SEQ < SND.NXT の判定
    }
}

impl PartialOrd for SequenceNumber {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.compare(*other))
    }
}

impl Ord for SequenceNumber {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.compare(*other)
    }
}
```

**実装ヒント**:
- シーケンス番号空間は循環的（0xFFFFFFFF → 0x00000000）
- 2^31 (約21億) を超える差は逆方向と判定
- RFC 793のアルゴリズム: `(i < j) = ((i < j) && (j - i < 2^31)) || ((i > j) && (i - j > 2^31))`

### Task A4: ISN生成

Initial Sequence Number（ISN）を生成します。

```rust
pub fn generate_isn() -> SequenceNumber {
    use std::time::{SystemTime, UNIX_EPOCH};

    // RFC 6528推奨: 時刻ベース + ランダム性
    // 簡略版: 現在時刻のマイクロ秒を使用
}
```

**実装ヒント**:
- RFC 6528（On the Generation of Secure ISNs）参照
- 予測困難なISNでセキュリティ向上
- 学習用では時刻ベースの簡略実装でOK

---

## Phase B: 送信バッファの実装

### Task B1: SendBuffer構造体の定義

送信待ちデータを管理する構造体を定義します。

```rust
pub struct SendBuffer {
    buffer: Vec<u8>,           // 送信待ちデータ
    next_seq: SequenceNumber,  // 次に送信するシーケンス番号
    unacked_seq: SequenceNumber, // 未確認の最小シーケンス番号
}
```

**実装ヒント**:
- `buffer`: アプリケーションから受け取ったデータ
- `next_seq`: 次に送信するバイトのシーケンス番号
- `unacked_seq`: ACKされていない最古のシーケンス番号
- `next_seq - unacked_seq` = 送信済み未確認データ量

### Task B2: データ追加メソッド

アプリケーションからのデータを送信バッファに追加します。

```rust
impl SendBuffer {
    pub fn new(initial_seq: SequenceNumber) -> Self {
        Self {
            buffer: Vec::new(),
            next_seq: initial_seq,
            unacked_seq: initial_seq,
        }
    }

    pub fn write(&mut self, data: &[u8]) -> Result<usize, String> {
        // データをバッファに追加
        // 追加したバイト数を返す
    }

    pub fn available_space(&self) -> usize {
        // 送信可能な残り容量
        // 実装例: 固定バッファサイズ - 現在のデータ量
    }
}
```

**実装ヒント**:
- `write`: データを`buffer`の末尾に追加
- バッファ容量制限を考慮（例: 64KB）
- 容量不足の場合はエラーまたは部分書き込み

### Task B3: データ取得メソッド

送信すべきデータをバッファから取得します。

```rust
impl SendBuffer {
    pub fn peek(&self, size: usize) -> &[u8] {
        // 次に送信するデータを取得（削除しない）
        // 最大sizeバイトまで
    }

    pub fn consume(&mut self, size: usize) {
        // 送信済みデータをマークして削除可能にする
        // next_seqを進める
    }
}
```

**実装ヒント**:
- `peek`: データを見るだけで削除しない
- `consume`: 送信完了後に呼び出し、`next_seq`を更新
- `buffer`からの削除は後で実装（ACK受信時）

### Task B4: ACK処理

ACK受信時に確認済みデータをバッファから削除します。

```rust
impl SendBuffer {
    pub fn acknowledge(&mut self, ack_seq: SequenceNumber) -> Result<usize, String> {
        // ack_seqまでのデータを確認済みとしてバッファから削除
        // 削除したバイト数を返す
    }

    pub fn unacked_data(&self) -> usize {
        // 送信済みだがACKされていないデータ量
        self.next_seq.wrapping_sub(self.unacked_seq) as usize
    }
}
```

**実装ヒント**:
- `acknowledge`: `ack_seq`が`unacked_seq`より大きい場合のみ処理
- 確認済みデータを`buffer`から削除
- `unacked_seq`を`ack_seq`に更新
- シーケンス番号の比較に注意（ラップアラウンド）

---

## Phase C: 受信バッファの実装

### Task C1: ReceiveBuffer構造体の定義

受信データを順序通りに管理する構造体を定義します。

```rust
pub struct ReceiveBuffer {
    buffer: Vec<u8>,           // 順序通りに並んだデータ
    next_expected: SequenceNumber, // 次に期待するシーケンス番号
    out_of_order: std::collections::BTreeMap<SequenceNumber, Vec<u8>>, // 順序外データ
}
```

**実装ヒント**:
- `buffer`: アプリケーションに渡す準備ができたデータ
- `next_expected`: 次に受信すべきシーケンス番号
- `out_of_order`: 順序が飛んで届いたデータを一時保管
- `BTreeMap`で順序を維持

### Task C2: データ受信メソッド

受信したセグメントをバッファに格納します。

```rust
impl ReceiveBuffer {
    pub fn new(initial_seq: SequenceNumber) -> Self {
        Self {
            buffer: Vec::new(),
            next_expected: initial_seq,
            out_of_order: std::collections::BTreeMap::new(),
        }
    }

    pub fn receive(&mut self, seq: SequenceNumber, data: &[u8]) -> Result<(), String> {
        // データを適切な位置に格納
        // 順序通りなら buffer に追加
        // 順序外なら out_of_order に保管
    }
}
```

**実装ヒント**:
- `seq == next_expected`: 順序通り → `buffer`に追加、`next_expected`を更新
- `seq > next_expected`: 順序外 → `out_of_order`に保管
- `seq < next_expected`: 重複データ → 無視またはエラー
- 順序通りデータ追加後、`out_of_order`をチェックして連続データを`buffer`に移動

### Task C3: データ読み取りメソッド

アプリケーションにデータを渡します。

```rust
impl ReceiveBuffer {
    pub fn read(&mut self, size: usize) -> Vec<u8> {
        // bufferから最大sizeバイトを取り出す
        // 取り出したデータはbufferから削除
    }

    pub fn available(&self) -> usize {
        // 読み取り可能なデータ量
        self.buffer.len()
    }
}
```

**実装ヒント**:
- `read`: `buffer`から指定バイト数を取り出して返す
- 取り出したデータは`buffer`から削除
- 要求サイズより少ない場合は利用可能な分だけ返す

### Task C4: 順序外データの処理

順序が飛んで届いたデータを適切に処理します。

```rust
impl ReceiveBuffer {
    fn process_out_of_order(&mut self) {
        // out_of_orderから連続するデータをbufferに移動
    }

    pub fn has_gap(&self) -> bool {
        // データにギャップがあるか確認
        !self.out_of_order.is_empty()
    }
}
```

**実装ヒント**:
- `process_out_of_order`: `next_expected`から連続するデータを探す
- 連続データを`buffer`に追加し、`out_of_order`から削除
- `next_expected`を更新
- 再帰的に処理（連続データがなくなるまで）

---

## Phase D: セグメント化ロジック

### Task D1: セグメント構造の定義

送信するデータセグメントの構造を定義します。

```rust
pub struct Segment {
    pub seq: SequenceNumber,
    pub data: Vec<u8>,
}

impl Segment {
    pub fn new(seq: SequenceNumber, data: Vec<u8>) -> Self {
        Self { seq, data }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}
```

**実装ヒント**:
- シンプルなデータセグメントの表現
- 実際のTCPヘッダーは別途構築（Step02参照）

### Task D2: MSSベースのセグメント化

Maximum Segment Size（MSS）に基づいてデータを分割します。

```rust
pub const DEFAULT_MSS: usize = 1460; // Ethernet MTU 1500 - IP header 20 - TCP header 20

pub fn segment_data(data: &[u8], mss: usize, start_seq: SequenceNumber) -> Vec<Segment> {
    // dataをmssサイズのセグメントに分割
    // 各セグメントに適切なシーケンス番号を割り当て
}
```

**実装ヒント**:
- MSS = MTU - IP header - TCP header
- データをmssサイズのチャンクに分割
- 各チャンクに連続したシーケンス番号を付与
- 最後のセグメントは残りのデータ（mss未満でOK）

### Task D3: セグメント検証

受信したセグメントの妥当性を検証します。

```rust
pub fn validate_segment(seg: &Segment, expected_seq: SequenceNumber, window_size: u32) -> bool {
    // セグメントが受信ウィンドウ内にあるか確認
    // expected_seq から window_size の範囲内か判定
}
```

**実装ヒント**:
- セグメントのシーケンス番号が受信ウィンドウ内にあるか確認
- `expected_seq <= seg.seq < expected_seq + window_size`
- ラップアラウンドを考慮した比較

---

## Phase E: データ送受信統合

### Task E1: TcpConnection構造体の定義

バッファと状態マシンを統合した接続構造を定義します。

```rust
use crate::step04::TcpStateMachine; // Step04の状態マシンを再利用

pub struct TcpConnection {
    state: TcpStateMachine,
    send_buffer: SendBuffer,
    recv_buffer: ReceiveBuffer,
    local_seq: SequenceNumber,
    remote_seq: SequenceNumber,
}
```

**実装ヒント**:
- Step04の状態マシンを組み込み
- 送受信バッファで実際のデータを管理
- `local_seq`: 自分の送信シーケンス番号
- `remote_seq`: 相手の送信シーケンス番号（自分の受信期待番号）

### Task E2: 送信処理の実装

データを送信バッファに書き込み、セグメント化して送信します。

```rust
impl TcpConnection {
    pub fn send(&mut self, data: &[u8]) -> Result<usize, String> {
        // 1. 状態チェック（Established等）
        // 2. 送信バッファに書き込み
        // 3. 送信可能なデータをセグメント化
        // 4. 実際の送信（後で実装）
    }
}
```

**実装ヒント**:
- `state.can_send_data()`で状態確認
- `send_buffer.write(data)`でバッファに追加
- セグメント化は後のタスクで実装
- この段階ではバッファへの書き込みまで

### Task E3: 受信処理の実装

受信したセグメントをバッファに格納し、アプリケーションに渡します。

```rust
impl TcpConnection {
    pub fn receive(&mut self, seq: SequenceNumber, data: &[u8]) -> Result<(), String> {
        // 1. 状態チェック
        // 2. シーケンス番号検証
        // 3. 受信バッファに格納
        // 4. ACK生成（後で実装）
    }

    pub fn read(&mut self, size: usize) -> Vec<u8> {
        // アプリケーションがデータを読み取る
        self.recv_buffer.read(size)
    }
}
```

**実装ヒント**:
- `state.can_receive_data()`で状態確認
- `validate_segment()`でセグメント検証
- `recv_buffer.receive()`でバッファに格納
- 順序通りのデータはアプリケーションに渡す準備

### Task E4: ACK生成と処理

受信確認の生成と処理を実装します。

```rust
impl TcpConnection {
    pub fn generate_ack(&self) -> SequenceNumber {
        // 次に期待するシーケンス番号を返す
        self.recv_buffer.next_expected
    }

    pub fn process_ack(&mut self, ack_seq: SequenceNumber) -> Result<(), String> {
        // ACK受信時の処理
        // 送信バッファから確認済みデータを削除
        self.send_buffer.acknowledge(ack_seq)
    }
}
```

**実装ヒント**:
- `generate_ack()`: 受信バッファの`next_expected`を返す
- `process_ack()`: 送信バッファの`acknowledge()`を呼び出す
- 累積ACK（それまでのすべてのデータを確認）

---

## Phase F: 統合テストと検証

### Task F1: 完全な送受信テスト

データの送信から受信、ACKまでの完全なフローをテストします。

**実装ヒント**:
- 小サイズデータ（100バイト）の送受信
- 順序通りの受信
- ACK処理とバッファクリア

### Task F2: セグメント化テスト

大きなデータのセグメント化を検証します。

**実装ヒント**:
- MSS × 3 のデータを送信
- 3つのセグメントに分割されることを確認
- 各セグメントのシーケンス番号が連続

### Task F3: 順序外受信テスト

データが順序通りに届かない場合の処理を検証します。

**実装ヒント**:
- セグメント3, 1, 2の順で受信
- 受信バッファで正しく並び替え
- アプリケーションには順序通りに渡される

### Task F4: バッファ境界テスト

バッファの容量制限を検証します。

**実装ヒント**:
- バッファ満杯時の書き込みエラー
- バッファ空時の読み取り
- 適切なエラーハンドリング

---

## 完成チェックリスト

- [ ] Phase A: シーケンス番号の定義と演算
- [ ] Phase B: 送信バッファの実装
- [ ] Phase C: 受信バッファの実装
- [ ] Phase D: セグメント化ロジック
- [ ] Phase E: データ送受信統合
- [ ] Phase F: すべてのテストがパス
- [ ] RFC 9293 Section 3.3, 3.7との対応を確認
- [ ] Step04の状態マシンとの統合を確認

## 参考リソース

- [RFC 9293 - Transmission Control Protocol](https://www.rfc-editor.org/rfc/rfc9293.html)
- RFC 9293 Section 3.3: Sequence Numbers
- RFC 9293 Section 3.7: Data Communication
- RFC 6528: On the Generation of Secure ISNs

## 次のステップ

Step05完了後は、Step06でACK処理とウィンドウサイズ管理を実装します。
