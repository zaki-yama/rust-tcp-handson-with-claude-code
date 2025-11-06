# Step03 `test_complete_handshake` のテスト方法

```bash
# Terminal 1
sudo ip netns exec host2 nc -l 10.0.1.1 40000

# Terminal 2
$ sudo ip netns exec host1 tcpdump -i any -n -v 'tcp'

# Terminal 3
$ sudo ip netns exec host1 /workspaces/rust-tcp/target/debug/deps/step03-ea0032633cd6533a test_complete_handshake --nocapture
```

### テストバイナリの特定方法

```bash
# 1. ビルド（変更があった場合のみ必要）
cargo test --bin step03 --no-run

# 2. バイナリパスを変数に保存
TEST_BIN=$(find target/debug/deps -name 'step03-*' -type f -executable | grep -v "\.d$")

# 3. 実行
sudo ip netns exec host1 $TEST_BIN test_complete_handshake --nocapture
```

## 結果

```bash
# Terminal 2
tcpdump: data link type LINUX_SLL2
tcpdump: listening on any, link-type LINUX_SLL2 (Linux cooked v2), snapshot length 262144 bytes
15:09:00.117238 host1-veth1 Out IP (tos 0x0, ttl 64, id 0, offset 0, flags [DF], proto TCP (6), length 40)
    10.0.0.1.39827 > 10.0.1.1.40000: Flags [S], cksum 0xcd46 (correct), seq 1761676485, win 8192, length 0
15:09:00.117407 host1-veth1 In  IP (tos 0x0, ttl 63, id 0, offset 0, flags [DF], proto TCP (6), length 44)
    10.0.1.1.40000 > 10.0.0.1.39827: Flags [S.], cksum 0x070b (correct), seq 716024016, ack 1761676486, win 64240, options [mss 1460], length 0
15:09:00.117484 host1-veth1 Out IP (tos 0x0, ttl 64, id 0, offset 0, flags [DF], proto TCP (6), length 40)
    10.0.0.1.39827 > 10.0.1.1.40000: Flags [.], cksum 0xf9b8 (correct), ack 1, win 8192, length 0
```



```bash
# Terminal 3
running 1 test
SYN sent: seq=1761676485
Received packet: 44 bytes, protocol=6
TCP packet: 10.0.1.1:40000 -> 10.0.0.1:39827
Successfully received packet after 1 attempts
ACK sent: seq=1761676486, ack=716024017
test tests::phase_f_tests::test_complete_handshake ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.00s
```
