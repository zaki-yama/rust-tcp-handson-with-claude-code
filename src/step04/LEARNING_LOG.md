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
