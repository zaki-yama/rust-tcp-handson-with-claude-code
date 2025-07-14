# Step 1 学習ログ

このファイルは、Step 1実装中に出た質問と回答の記録です。

## 目次
1. [構造体の実装](#構造体の実装)
2. [Raw Socket関数の実装](#raw-socket関数の実装)
3. [IP_HDRINCLオプション](#ip_hdrincl-オプション)
4. [IPヘッダーフィールド初期化](#ipヘッダーフィールド初期化)
5. [エンディアンとto_be()](#エンディアンとto_be)

---

## 構造体の実装

### Q: IpHeaderの実装のところ、何をしたらいいのか全然わからない

**A: IPヘッダー構造体の実装ガイド**

RFC 791のIPヘッダー構造に基づいて、以下の構造体を実装：

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

**各フィールドの詳細**:

| フィールド | サイズ | 説明 |
|------------|--------|------|
| Version + IHL | 1バイト | バージョン(4bit) + ヘッダー長(4bit) |
| Type of Service | 1バイト | サービスタイプ |
| Total Length | 2バイト | パケット全体の長さ |
| Identification | 2バイト | パケット識別子 |
| Flags + Fragment Offset | 2バイト | フラグ(3bit) + フラグメントオフセット(13bit) |
| Time to Live | 1バイト | TTL |
| Protocol | 1バイト | 上位プロトコル (TCP=6) |
| Header Checksum | 2バイト | ヘッダーチェックサム |
| Source Address | 4バイト | 送信元IPアドレス |
| Destination Address | 4バイト | 宛先IPアドレス |

**実装のポイント**:
- `#[repr(C, packed)]`でC言語互換、パディングなし
- ネットワークバイトオーダー（big-endian）への変換が必要
- エンディアン変換: `.to_be()`メソッド使用

---

## Raw Socket関数の実装

### Q: create_new_socketの具体的な実装方法が分からない

**A: create_raw_socket関数の実装**

```rust
fn create_raw_socket() -> Result<i32, Box<dyn std::error::Error>> {
    // 1. raw socket作成
    let socket_fd = unsafe { 
        libc::socket(libc::AF_INET, libc::SOCK_RAW, libc::IPPROTO_TCP) 
    };
    
    // エラーチェック
    if socket_fd < 0 {
        return Err("Failed to create raw socket".into());
    }
    
    // 2. IP_HDRINCLオプション設定
    let optval: i32 = 1;  // 1 = 有効化
    let result = unsafe {
        libc::setsockopt(
            socket_fd,                                    // ソケットFD
            libc::IPPROTO_IP,                            // IPプロトコルレベル
            libc::IP_HDRINCL,                            // IP_HDRINCLオプション
            &optval as *const _ as *const libc::c_void,  // 値へのポインタ
            std::mem::size_of::<i32>() as u32,           // 値のサイズ
        )
    };
    
    // エラーチェック
    if result < 0 {
        unsafe { libc::close(socket_fd); }  // ソケットクローズ
        return Err("Failed to set IP_HDRINCL".into());
    }
    
    Ok(socket_fd)
}
```

---

## IP_HDRINCLオプション

### Q: IP_HDRINCLオプションの設定部分が何をしているか分からない

**A: IP_HDRINCLオプションの詳細説明**

### 目的
OSに「IPヘッダーは自分で作るから、自動生成しないで」と伝える

### なぜ必要？
- 通常、OSがIPヘッダーを自動で作成・付与
- Raw socketで**手作りIPヘッダー**を送りたい場合に必要
- このオプションがないと、OSが作ったヘッダー + あなたのヘッダー = 重複

### setsockopt呼び出しの詳細

```rust
let result = unsafe {
    libc::setsockopt(
        socket_fd,                                  // ソケットFD
        libc::IPPROTO_IP,                           // IPプロトコルレベル
        libc::IP_HDRINCL,                           // IP_HDRINCLオプション
        &optval as *const _ as *const libc::c_void, // 値へのポインタ
        std::mem::size_of::<i32>() as u32,          // 値のサイズ
    )
};
```

**各引数の説明**:

1. **socket_fd**: 設定対象のソケット
2. **IPPROTO_IP**: IPプロトコルレベルでの設定
3. **IP_HDRINCL**: 設定したいオプション名
4. **&optval**: 設定値（1=有効、0=無効）
5. **size**: 設定値のバイト数

**ポインタ変換の詳細**:
```rust
let optval: i32 = 1;
&optval                           // optvalのRust参照
as *const _                       // 型推論でconst raw pointer
as *const libc::c_void           // Cのvoid*型に変換
```

**結果**:
- 成功時: 0
- 失敗時: -1（エラー）

---

## IPヘッダーフィールド初期化

### Q: IpHeaderのそれぞれの値は何で初期化すべき？

**A: 各フィールドの適切な初期化値**

```rust
impl IpHeader {
    fn new(source: Ipv4Addr, dest: Ipv4Addr, data_len: u16) -> Self {
        let total_length = IP_HEADER_SIZE as u16 + data_len;
        
        Self {
            version_ihl: 0x45,                    // IPv4, 20バイトヘッダー
            tos: 0,                               // 通常サービス
            length: total_length.to_be(),         // 全体長
            id: 0x1234_u16.to_be(),              // 識別子（固定）
            flags_fragment: 0x4000_u16.to_be(),  // Don't Fragment
            ttl: 64,                             // ホップ制限
            protocol: 6,                         // TCP
            checksum: 0,                         // 後で計算
            source: u32::from(source).to_be(),   // 送信元IP
            destination: u32::from(dest).to_be(), // 宛先IP
        }
    }
}
```

**各フィールドの詳細**:

1. **version_ihl: 0x45**
   - Version (4bit): `4` (IPv4)
   - IHL (4bit): `5` (20バイト = 5 × 4バイト)

2. **tos: 0**
   - 通常は0（特別なサービス要求なし）

3. **length: (20 + data_len).to_be()**
   - IPヘッダー(20バイト) + データ長

4. **id: 0x1234_u16.to_be()**
   - フラグメンテーション時の識別用（今回は固定値）

5. **flags_fragment: 0x4000_u16.to_be()**
   - Flags (3bit): `010` = Don't Fragment
   - Fragment Offset (13bit): `0`

6. **ttl: 64**
   - Linux/macOSのデフォルト値

7. **protocol: 6**
   - TCP = 6, UDP = 17, ICMP = 1

8. **checksum: 0**
   - 初期値（後でcalculate_checksum()で計算）

---

## エンディアンとto_be()

### Q: to_be()している理由と何をしているか教えて

**A: エンディアンとネットワークバイトオーダーの説明**

### エンディアンとは
**コンピューターがマルチバイトデータを格納する順序**

### ホストバイトオーダー vs ネットワークバイトオーダー

```rust
let value = 0x1234u16;

// リトルエンディアン（多くのCPU）
// メモリ: [0x34, 0x12]

// ビッグエンディアン（ネットワーク標準）  
// メモリ: [0x12, 0x34]
```

### to_be()の動作

```rust
let host_value = 0x1234u16;
let network_value = host_value.to_be();

// x86_64（リトルエンディアン）の場合
// host_value    = 0x1234 (メモリ: [0x34, 0x12])
// network_value = 0x1234 (メモリ: [0x12, 0x34]) ← 変換された

// ビッグエンディアンCPUの場合
// 変換不要なのでそのまま
```

### なぜネットワークバイトオーダーが必要？

1. **統一性**: 異なるCPU間での通信を可能に
2. **RFC標準**: すべてのネットワークプロトコルで使用
3. **相互運用性**: どんな機器とも通信できる

### IPヘッダーフィールドの詳細定義

#### 1. version_ihl: u8 (1バイト)
```
|7 6 5 4|3 2 1 0|
|Version|  IHL  |
```
- **Version (4bit)**: IPプロトコルバージョン（IPv4 = 4）
- **IHL (4bit)**: Internet Header Length（ヘッダー長を4バイト単位で表現）
- **0x45の意味**: `0100 0101` = Version 4, IHL 5 (20バイト)

#### 2. flags_fragment: u16 (2バイト)
```
|15 14 13|12 11 10 9 8 7 6 5 4 3 2 1 0|
| Flags  |      Fragment Offset        |
```
**Flags (3bit)**:
- bit 0: 予約（0）
- bit 1: DF (Don't Fragment) = 1
- bit 2: MF (More Fragments) = 0

**0x4000の意味**: `0100 0000 0000 0000` = DF=1, オフセット=0

### 実装での注意点

```rust
// ❌ 間違い - ホストバイトオーダーのまま
id: 0x1234_u16,

// ✅ 正しい - ネットワークバイトオーダーに変換
id: 0x1234_u16.to_be(),
```

---

## 学習のポイント

1. **システムコールの理解**: Rustから直接Cのシステムコールを呼ぶ方法
2. **メモリレイアウト**: パケットデータのバイト配列での表現
3. **エンディアン**: ネットワークバイトオーダー（ビッグエンディアン）の重要性
4. **エラーハンドリング**: システムレベルでのエラー処理
5. **RFC準拠**: 標準仕様に基づいた実装の重要性

---

## RFCとチェックサムアルゴリズム

### Q: RFC1071というものが初めて登場しましたが、これはTCPプロトコル、ならびにRFC791とはどういう関係ですか？

**A: RFC間の関係とチェックサム仕様の歴史**

### RFC文書の関係性

#### RFC 791 (Internet Protocol - 1981年)
- **役割**: IPv4プロトコルの基本仕様
- **チェックサム記述**: 概要レベルの説明のみ
- **原文**: 
  > "The checksum field is the 16 bit one's complement of the one's complement sum of all 16 bit words in the header."

#### RFC 1071 (Computing the Internet Checksum - 1988年)
- **役割**: インターネットチェックサムの詳細実装ガイド
- **関係**: RFC 791の補完文書
- **目的**: チェックサム計算の具体的アルゴリズムと実装例を提供

### なぜ別RFCになったのか？

1. **役割分担**:
   - RFC 791: プロトコル仕様（何をするか）
   - RFC 1071: 実装詳細（どうやってするか）

2. **時代的背景**:
   - 1981年: IPv4策定時は概要のみで十分
   - 1988年: 実装が増えて詳細ガイドが必要に

3. **対象範囲**:
   - RFC 1071は**IP、TCP、UDP**すべてで使用される共通アルゴリズム

### TCPとの関係

**TCPでも同じチェックサムアルゴリズム使用**:
- **IPヘッダー**: RFC 791 + RFC 1071
- **TCPヘッダー**: RFC 9293 + RFC 1071
- **UDPヘッダー**: RFC 768 + RFC 1071

### 実装上の位置づけ

```
プロトコル階層:
┌─────────────┐
│   TCP/UDP   │ ← RFC 9293/768 (プロトコル) + RFC 1071 (チェックサム)
├─────────────┤
│     IP      │ ← RFC 791 (プロトコル) + RFC 1071 (チェックサム)
├─────────────┤
│  Ethernet   │
└─────────────┘
```

### 学習での扱い

**Step 1では**:
- **主要**: RFC 791 (IPヘッダー構造)
- **補助**: RFC 1071 (チェックサム実装)

**Step 2以降では**:
- **主要**: RFC 9293 (TCP仕様)
- **補助**: RFC 1071 (TCPチェックサム実装)

### RFC 1071の重要性

**なぜ重要？**
1. **実装精度**: 正確なチェックサム計算
2. **性能最適化**: 効率的なアルゴリズム
3. **相互運用性**: 標準準拠の実装

**現在のStep 1実装では**、RFC 1071のアルゴリズムに従ってIPヘッダーチェックサムを実装します。

---

## 次のステップ

以下の関数をまだ実装する必要があります：

1. `IpHeader::calculate_checksum()` - チェックサム計算（RFC 1071準拠）
2. `IpHeader::to_bytes()` - バイト配列変換
3. `send_packet()` - パケット送信
4. `receive_packet()` - パケット受信
5. `parse_ip_header()` - ヘッダー解析
6. `main()` - メイン処理