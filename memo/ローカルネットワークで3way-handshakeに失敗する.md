# ローカルネットワークで3way-handshakeに失敗する

## ステップ

Step03 Task F1: 完全handshakeテスト

## 事象

### 試行1

受信側 (nc)

```
$ nc -l 8080
```

tcpdump

```
$ sudo tcpdump -i lo0 port 8080
```

pfctl

```bash
$ echo "block return out proto tcp from any to any port 8080 flags R" > /tmp/pf_rules.conf
$ sudo pfctl -f /tmp/pf_rules.conf
$ sudo pfctl -e  # pfを有効化
```

```
$ sudo cargo test --bin step03 phase_f_tests::test_complete_handshake
```

### 1の結果

tcpdump のログ

```
00:25:53.293428 IP (tos 0x0, ttl 64, id 58285, offset 0, flags [DF], proto TCP (6), length 40, bad cksum 0 (->166e)!)
    192.168.1.11.56547 > localhost.http-alt: Flags [S], cksum 0xda1b (correct), seq 1759776698, win 8192, length 0
00:25:53.293496 IP (tos 0x0, ttl 64, id 0, offset 0, flags [DF], proto TCP (6), length 44, bad cksum 0 (->fa17)!)
    localhost.http-alt > 192.168.1.11.56547: Flags [S.], cksum 0x40d3 (incorrect -> 0x4f1d), seq 6052017, ack 1759776699, win 65535, options [mss 16344], length 0
00:25:53.293506 IP (tos 0x0, ttl 64, id 0, offset 0, flags [DF], proto TCP (6), length 40, bad cksum 0 (->fa1b)!)
    192.168.1.11.56547 > localhost.http-alt: Flags [R], cksum 0x40cf (incorrect -> 0xfa18), seq 1759776699, win 0, length 0
```



### 1の分析 by Gemini

NOTE: ↑のログと別のタイミングで実行しているので、ログの内容は微妙に違う。

**パケット1：送信SYNパケット**

```
06:48:53.972152 IP (...) 192.168.1.11.64353 > localhost.http-alt: Flags, cksum 0x856a (correct), seq 1758676478, win 8192, length 0
```


- 分析: この最初のパケットは、ユーザーのRustアプリケーションによって生成され、送信されたTCPセグメントです。これは3ウェイハンドシェイクの開始を告げるSYNパケットであり、以下の点で正しく構成されています。
  - Flags: SYNフラグが正しくセットされています。
  - seq 1758676478: 有効な初期シーケンス番号（ISN）が設定されています。
  - 192.168.1.11.64353 > localhost.http-alt: 送信元IP/ポートから宛先IP/ポート（127.0.0.1:8080）への方向が正しいです。
- 重要な所見: tcpdumpがcksum 0x856a (correct)と注釈している点は特に重要です。これは、アプリケーションが算出したTCPチェックサムが正しいことを示しており、TCPヘッダと（この場合は存在しない）ペイロードに対するチェックサム計算ロジックが、この最初のパケットに関しては正しく実装されている可能性が高いことを示唆しています。

**パケット2：受信SYN-ACKパケットとチェックサムの偽陽性**

```
06:48:53.972280 IP (...) localhost.http-alt > 192.168.1.11.64353: Flags, cksum 0x40d3 (incorrect -> 0x4fd9), seq 1318958338, ack 1758676479, win 65535, options [mss 16344], length 0
```

- 分析: このパケットは、netcatで待機しているサーバーからの正当な応答です。
  - Flags: SYNフラグとACKフラグの両方がセットされています。
  - ack 1758676479: パケット1のシーケンス番号（1758676478）に1を加えた値であり、クライアントのSYNを正しく確認応答（ACK）しています。
  - seq 1318958338: サーバー自身のISNを通知しています。
- 「bad cksum」と「incorrect」の異常: ログにはIPヘッダにbad cksum、TCPヘッダにincorrectという警告が表示されています。これは、問題の診断においてしばしば誤解を招くポイントですが、実際のエラーではありません。特にループバックインターフェース（lo0）上の通信では、OSカーネルがパフォーマンス最適化のためにチェックサムの計算を省略したり、ハードウェア（NIC）にオフロードしたりすることが一般的です 1。
  tcpdumpは、このチェックサムがNICによって計算・挿入される前の段階でパケットをキャプチャするため、チェックサムが不正であるかのように見えてしまいます 3。したがって、この警告は今回の問題の原因ではなく、無視して問題ありません。

**パケット3：接続の終焉を告げるカーネルのRSTパケット**

```
06:48:53.972303 IP (...) 192.168.1.11.64353 > localhost.http-alt: Flags, cksum 0x40cf (incorrect -> 0xa567), seq 1758676479, win 0, length 0
```

- 分析: これが、ハンドシェイクを失敗させ、タイムアウトを引き起こしている直接の原因となるパケットです。
  - Flags: RST（リセット）フラグがセットされており、接続を強制的に中断することを示します。
  - 送信元: 送信元IPアドレスとポート（192.168.1.11.64353）は、ユーザーのアプリケーションのものと一致しています。しかし、このパケットはアプリケーション自身が送信したものではありません。
  - シーケンス番号: seq 1758676479というシーケンス番号は、パケット2でサーバーが期待していたACK番号と一致しています。これにより、このRSTはサーバーから見て正当なものとして受け入れられ、接続は確立されることなく破棄されます。
- 結論: このRSTパケットは、macOSのカーネルによって生成され、ネットワークスタックに注入されたものです。


---

よく見ると、

```
sudo pfctl -f /tmp/pf_rules.conf

pfctl: Use of -f option, could result in flushing of rules
present in the main ruleset added by the system at startup.
See /etc/pf.conf for further details.

No ALTQ support in kernel
ALTQ related functions disabled
/tmp/pf_rules.conf:1: syntax error
pfctl: Syntax error in config file: pf rules not loaded
```


## 試行2

pfctl のルールを修正する

```bash
echo "block drop out quick on lo0 proto tcp from any to 127.0.0.1 port 8080 flags R/R" > /tmp/pf_rules.conf
```

```bash
# 構文チェック
sudo pfctl -vnf /tmp/pf_rules.conf
# ロード
sudo pfctl -f /tmp/pf_rules.conf

# 有効化
sudo pfctl -e

# ロード確認
sudo pfctl -sr
```

## 2の結果

```
01:26:48.765198 IP (tos 0x0, ttl 64, id 34588, offset 0, flags [DF], proto TCP (6), length 40, bad cksum 0 (->72ff)!)
    192.168.1.11.57202 > localhost.http-alt: Flags [S], cksum 0xc945 (correct), seq 1759780353, win 8192, length 0
01:26:48.765277 IP (tos 0x0, ttl 64, id 0, offset 0, flags [DF], proto TCP (6), length 44, bad cksum 0 (->fa17)!)
    localhost.http-alt > 192.168.1.11.57202: Flags [S.], cksum 0x40d3 (incorrect -> 0x8138), seq 1139135030, ack 1759780354, win 65535, options [mss 16344], length 0
01:26:49.766522 IP (tos 0x0, ttl 64, id 0, offset 0, flags [DF], proto TCP (6), length 44, bad cksum 0 (->fa17)!)
    localhost.http-alt > 192.168.1.11.57202: Flags [S.], cksum 0x40d3 (incorrect -> 0x8138), seq 1139135030, ack 1759780354, win 65535, options [mss 16344], length 0
01:26:51.766573 IP (tos 0x0, ttl 64, id 0, offset 0, flags [DF], proto TCP (6), length 44, bad cksum 0 (->fa17)!)
    localhost.http-alt > 192.168.1.11.57202: Flags [S.], cksum 0x40d3 (incorrect -> 0x8138), seq 1139135030, ack 1759780354, win 65535, options [mss 16344], length 0
01:26:55.767476 IP (tos 0x0, ttl 64, id 0, offset 0, flags [DF], proto TCP (6), length 44, bad cksum 0 (->fa17)!)
    localhost.http-alt > 192.168.1.11.57202: Flags [S.], cksum 0x40d3 (incorrect -> 0x8138), seq 1139135030, ack 1759780354, win 65535, options [mss 16344], length 0
01:27:03.769005 IP (tos 0x0, ttl 64, id 0, offset 0, flags [DF], proto TCP (6), length 44, bad cksum 0 (->fa17)!)
    localhost.http-alt > 192.168.1.11.57202: Flags [S.], cksum 0x40d3 (incorrect -> 0x8138), seq 1139135030, ack 1759780354, win 65535, options [mss 16344], length 0
01:27:19.769992 IP (tos 0x0, ttl 64, id 0, offset 0, flags [DF], proto TCP (6), length 44, bad cksum 0 (->fa17)!)
    localhost.http-alt > 192.168.1.11.57202: Flags [S.], cksum 0x40d3 (incorrect -> 0x8138), seq 1139135030, ack 1759780354, win 65535, options [mss 16344], length 0
01:27:51.770390 IP (tos 0x0, ttl 64, id 0, offset 0, flags [DF], proto TCP (6), length 44, bad cksum 0 (->fa17)!)
    localhost.http-alt > 192.168.1.11.57202: Flags [S.], cksum 0x40d3 (incorrect -> 0x8138), seq 1139135030, ack 1759780354, win 65535, options [mss 16344], length 0
01:28:03.765954 IP (tos 0x0, ttl 64, id 0, offset 0, flags [DF], proto TCP (6), length 40, bad cksum 0 (->fa1b)!)
    localhost.http-alt > 192.168.1.11.57202: Flags [R.], cksum 0x40cf (incorrect -> 0xd315), seq 1, ack 1, win 65535, length 0
```

## 2の分析 by Gemini

> お客様のRustコード内で、ACKパケットを送信する直前のチェックサム計算ロジックを再確認してください。具体的には、IP擬似ヘッダを正しく構築し、それをTCPヘッダと連結した上でチェックサムを計算しているか、という点に焦点を当ててデバッグを進めることを強くお勧めします。


と言っていたが、fn receive_packet_timeout のコードを提示すると

> デバッグ出力が常にNo data availableであることから、新たな問題点が明確になりました。以前はカーネルがRSTパケットを送信していましたが、pfctlルールによってその問題は解決しました。現在の問題は、お客様のアプリケーションが、サーバーから送信されているはずのSYN-ACKパケットを全く受信できていないことにあります。

-> `bind` しろと言われる
