# Step04 Learning Log

## TcpEventとRFC 9293の対応関係

### 質問内容

enum TcpEventの各値は、RFCのどこと対応づいていますか？

### 回答

#### User Calls（アプリケーションからの要求）

| TcpEvent | RFC対応 | RFC箇所 |
|----------|---------|---------|
| `Connect` | **active OPEN** | Section 3.10.1 (OPEN Call)<br>Figure 5: "active OPEN" |
| `Listen` | **passive OPEN** | Section 3.10.1 (OPEN Call)<br>Figure 5: "passive OPEN" |
| `Close` | **CLOSE** | Section 3.10.4 (CLOSE Call)<br>Figure 5: "CLOSE" |

**補足**:
- RFCでは`OPEN`は1つだが、active/passiveのモードがある
- Step04では分かりやすく`Connect`と`Listen`に分割

#### Segment Arrives（パケット受信）

| TcpEvent | RFC対応 | RFC箇所 |
|----------|---------|---------|
| `ReceiveSyn` | **rcv SYN** | Figure 5: "rcv SYN"<br>Section 3.10.7.2: LISTEN STATE - "Third, check for a SYN"<br>Section 3.10.7.3: SYN-SENT STATE - "Fourth, check the SYN bit" |
| `ReceiveSynAck` | **rcv SYN,ACK** | Figure 5: "rcv SYN,ACK"<br>Section 3.10.7.3: SYN-SENT STATE - SYN+ACK処理 |
| `ReceiveAck` | **rcv ACK of SYN**<br>**rcv ACK of FIN** | Figure 5: "rcv ACK of SYN", "rcv ACK of FIN"<br>Section 3.10.7: 各状態でのACK処理 |
| `ReceiveFin` | **rcv FIN** | Figure 5: "rcv FIN"<br>Section 3.10.7.4以降: 各状態でのFIN処理 |
| `ReceiveRst` | **rcv RST** | Figure 5: "rcv RST"<br>Section 3.10.7: 各状態でのRST処理 |

**補足**:
- Section 3.10.7は**状態ごと**にセグメント処理を記述（3.10.7.1 CLOSED, 3.10.7.2 LISTEN, 3.10.7.3 SYN-SENT...）
- 各イベント（SYN, ACK, FIN, RST）は、現在の状態によって異なる処理が行われる
- Figure 5は簡潔な表記で状態遷移を示し、Section 3.10.7が詳細なロジックを記述

#### Timeouts（タイマー）

| TcpEvent | RFC対応 | RFC箇所 |
|----------|---------|---------|
| `Timeout` | **Timeout=2MSL** | Section 3.10.8 (Timeouts)<br>Figure 5: "Timeout=2MSL" |

**補足**:
- RFCには他のタイムアウトもある（再送タイムアウトなど）
- Step04では主にTIME-WAITからの遷移用

#### RFCで定義されているが未実装のイベント

| RFCイベント | 理由 |
|-----------|------|
| **SEND** | Step05以降で実装（データ送信） |
| **RECEIVE** | Step05以降で実装（データ受信） |
| **ABORT** | 未実装（RSTで代用） |
| **STATUS** | 未実装（状態問い合わせ） |
| **再送タイムアウト** | Step07以降で実装 |

#### まとめ

`TcpEvent`は、RFC 9293の以下に基づいています：
- **Section 3.10**: Event Processing（全体的な定義）
- **Figure 5**: TCP Connection State Diagram（状態遷移図の矢印ラベル）

Step04では状態遷移に**直接関係するイベントのみ**を抽出して実装しています。

---

## アクティブクローズ／パッシブクローズの定義とRFC対応

### 質問内容

アクティブクローズ、パッシブクローズの定義はRFCのどこに記述がありますか？

### 回答

#### RFC 9293における用語の定義

**✅ 正式にRFCで定義されている用語**

**Active Open / Passive Open** は **RFC 9293 Section 3.5** で明確に定義されています：

1. **Active Open** (アクティブオープン)
   - 「接続確立を能動的に追求する」
   - クライアント側の動作
   - Section 3.5: "connection establishment is to be actively pursued"

2. **Passive Open** (パッシブオープン)
   - 「接続要求を受け入れる準備をする」
   - サーバー側の動作
   - Section 3.5: "A passive OPEN request means that the process wants to accept incoming connection requests"

**⚠️ RFCには出てこない用語**

**Active Close / Passive Close** という用語は、実は **RFC 9293には正式に定義されていません**。

RFCでは、**Section 3.6 (Closing a Connection)** で以下の3つのケースとして記述されています：

1. **Case 1: Local user initiates CLOSE**
   - ローカルユーザーが終了を開始する
   - （一般的に「アクティブクローズ」と呼ばれるもの）
   - Figure 12の "TCP Peer A"

2. **Case 2: Remote TCP sends FIN**
   - リモートTCPがFINを送信して終了を開始
   - （一般的に「パッシブクローズ」と呼ばれるもの）
   - Figure 12の "TCP Peer B"

3. **Case 3: Both sides close simultaneously**
   - 両側が同時に終了（同時クローズ）

#### まとめ

| 用語 | RFC 9293での状態 | 説明 |
|------|------------------|------|
| **Active Open** | ✅ 正式な用語 | Section 3.5で定義 |
| **Passive Open** | ✅ 正式な用語 | Section 3.5で定義 |
| **Active Close** | ❌ 非公式（教育的慣用表現） | RFC ではCase 1として記述 |
| **Passive Close** | ❌ 非公式（教育的慣用表現） | RFC ではCase 2として記述 |

#### RFCを正確に参照するなら

- **接続確立**: RFC 9293 **Section 3.5** "Establishing a Connection"
  - Active Open / Passive Open の定義

- **接続終了**: RFC 9293 **Section 3.6** "Closing a Connection"
  - Case 1, 2, 3 による終了シナリオ
  - Figure 12, 13 に終了シーケンスの図

- **状態遷移全体**: RFC 9293 **Section 3.3.2** "State Machine Overview" と **Figure 5**

#### 結論

Step04のREADME.mdで「アクティブクローズ」「パッシブクローズ」という用語を使っているのは、Active/Passive Openとの対称性を持たせて理解しやすくするための**教育的な表現**です。RFCを読む際は、正式な用語と教育的な慣用表現を区別することが重要です。

---

## 「4-way handshake」用語のRFC対応

### 質問内容

README.md:302に「4-way handshake」とありますが、この用語はRFCと照らし合わせて正式な用語ですか？

### 回答

#### RFC 9293における「handshake」用語の使用状況

**✅ 正式にRFCで定義されている用語**

**"three-way handshake" (3WHS)** - **正式な用語**
- RFC 9293全体で使われている
- Section 3.5: "The 'three-way handshake' is the procedure used to establish a connection."
- Figure 6に図解あり
- 接続確立のSYN → SYN-ACK → ACKのシーケンス

**❌ RFCには出てこない用語**

**"four-way handshake" (4-way handshake)** - **RFC 9293には出てこない**
- 接続終了のシーケンスを表す用語として、教育的には広く使われている
- しかし、RFC 9293ではこの用語は使われていない

#### RFCでの接続終了の記述方法

RFC 9293 **Section 3.6 "Closing a Connection"** では：

1. 「CLOSE」という操作の説明
2. 3つのケース（Case 1, 2, 3）による分類
3. FINとACKのシーケンスは **Figure 12, 13** で図示
4. "simplex fashion"（単方向的）として扱う、という説明

つまり、**メッセージ数や「handshake」という用語を使わずに、状態遷移とシーケンスで説明**しています。

#### まとめ

| 用語 | RFC 9293での状態 | 実際の用途 |
|------|------------------|-----------|
| **three-way handshake** | ✅ 正式な用語 | 接続確立（SYN, SYN-ACK, ACK） |
| **four-way handshake** | ❌ 非公式（教育的慣用表現） | 接続終了（FIN, ACK, FIN, ACK） |

#### なぜ「4-way handshake」と呼ばれるのか？

教育的には以下の対称性から理解しやすいため、広く使われています：

**接続確立（3-way handshake）**:
```
Client → Server: SYN
Server → Client: SYN-ACK
Client → Server: ACK
```

**接続終了（4-way handshakeと呼ばれる）**:
```
側A → 側B: FIN
側B → 側A: ACK
側B → 側A: FIN
側A → 側B: ACK
```

#### RFC準拠の正確な表現

RFCに準拠した正確な表現は以下のとおり：
- ✅ "Connection closing sequence"
- ✅ "FIN exchange"
- ✅ "Closing a Connection (Section 3.6)"
- ❌ "4-way handshake" (RFCには出てこない)

#### 結論

Step04 README.md:302の「4-way handshake」は、**教育的にわかりやすくするための慣用表現**です。3-way handshakeとの対比で理解を助けるために使っています。実装においては、RFC 9293 Section 3.6とFigure 5の状態遷移を正しく実装できていれば、用語の違いは問題になりません。

---

## RFCにおけるタイムアウトの記述

### 質問内容

タイムアウトについて言及されているのはRFCのどこでしょうか

### 回答

#### RFC 9293で明示的に記載されているタイムアウト

**RFC 9293 Figure 5 (TCP Connection State Diagram)** では、明示的に記載されているタイムアウト遷移は**1つだけ**です:

```
TIME-WAIT → CLOSED (Timeout=2MSL)
```

**詳細**:
- **2MSL**: Maximum Segment Lifetime の2倍
- **MSL**: RFC 9293では2分と定義されている（Section 3.3.2）
- **目的**: 接続終了後、ネットワーク上の遅延パケットが消滅するのを待つ
- **記述箇所**: Figure 5の状態遷移図で明示

**TIME-WAIT状態の説明**（RFC 9293 Section 3.3.2より）:
> "TIME-WAIT represents waiting for enough time to pass to be sure the remote TCP peer received the acknowledgment of its connection termination request and to avoid new connections being impacted by delayed segments from previous connections."

#### 暗黙的なタイムアウト処理

**SYN-SENT**や**SYN-RECEIVED**状態からのタイムアウト遷移は、RFC 9293 Figure 5の状態遷移図には**明示的に記載されていません**。

しかし、実装上は以下のセクションで関連する記述があります:

1. **Section 3.8.1 - Retransmission Timeout (RTO)**
   - 再送タイムアウトの概念を説明
   - 具体的なアルゴリズムはRFC 6298を参照

2. **Section 3.8.3 - TCP Connection Failures**
   - 接続失敗時の動作
   - 再送試行回数の限界に達した場合の処理

3. **Section 3.10.8 - Timeouts**
   - タイムアウト処理全般の説明
   - （提供されたRFC抜粋には詳細が含まれていなかった）

#### 実際のTCP実装で必要なタイムアウト

実際のTCP実装では、以下のタイムアウトが必要です:

**接続確立フェーズ**:
- **SYN-SENT状態**: SYN-ACKが返ってこない
  - 複数回の再送試行（指数バックオフ）
  - 最終的にCLOSEDへ遷移

- **SYN-RECEIVED状態**: ACKが返ってこない
  - 複数回のSYN-ACK再送
  - 最終的にLISTENまたはCLOSEDへ遷移

**接続終了フェーズ**:
- **TIME-WAIT状態**: 2MSLタイマー（RFC明示）
  - タイムアウト後にCLOSEDへ遷移

**データ転送フェーズ**（Step05以降）:
- 再送タイムアウト（RTO）
- Keep-aliveタイマー
- User Timeout（RFC 5482）

#### Step04での扱い

Step04のREADME.mdとテストコードでは、**学習目的で簡略化**しています:

```rust
// 実装が期待される動作（tests.rsより）
SynSent + Timeout → Closed          // 簡略化
SynReceived + Timeout → Closed      // 簡略化
TimeWait + Timeout → Closed         // RFC準拠
```

これは以下の理由による簡略化です:
- 再送ロジックはStep07で実装予定
- 状態遷移の本質的な理解に集中
- 複雑な再送アルゴリズムを避ける

#### まとめ

| タイムアウト種別 | RFC 9293 Figure 5 | 実装上の必要性 | Step04での扱い |
|---------------|-------------------|--------------|--------------|
| TIME-WAIT → CLOSED | ✅ 明示的に記載 | 必須 | 実装する |
| SYN-SENT → CLOSED | ❌ 図には未記載 | 実装上必要 | 簡略版を実装 |
| SYN-RECEIVED → CLOSED | ❌ 図には未記載 | 実装上必要 | 簡略版を実装 |
| 再送タイムアウト | Section 3.8.1参照 | 必須 | Step07で実装予定 |

**結論**: RFC 9293のFigure 5で明示されているタイムアウトは**TIME-WAIT→CLOSED（2MSL）のみ**です。接続確立時のタイムアウトは、再送ロジック（Section 3.8.1, RFC 6298）の一部として扱われており、状態遷移図には明示されていません。Step04では学習目的で簡略化して実装します。
