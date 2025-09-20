use super::*;
use std::time::{Duration, Instant};

// =============================================================================
// Phase A: 3-way handshakeの理解と設計 - TDD Tests
// =============================================================================

#[cfg(test)]
mod phase_a_tests {
    use super::*;

    // Task A3: 基本状態定義のテスト
    #[test]
    fn test_tcp_state_enum_basic() {
        // Red: 最初は失敗する（状態enumが未定義）
        let state = TcpState::Closed;
        assert_eq!(state, TcpState::Closed);
        assert_ne!(state, TcpState::SynSent);
        assert_ne!(state, TcpState::Established);

        // Clone, PartialEqが実装されているかテスト
        let cloned_state = state.clone();
        assert_eq!(state, cloned_state);
    }

    // Task A4: TcpConnection基本構造のテスト
    #[test]
    fn test_tcp_connection_creation() {
        // TcpConnection::new実装後にテストを有効化
        let remote_ip = Ipv4Addr::new(127, 0, 0, 1);
        let conn = TcpConnection::new(remote_ip, 80).unwrap();

        // 初期状態の確認
        assert_eq!(conn.state, TcpState::Closed);
        assert!(!conn.is_connected());
        assert_eq!(conn.remote_ip, remote_ip);
        assert_eq!(conn.remote_port, 80);
    }
}

// =============================================================================
// Phase B: 基本構造の実装 - TDD Tests
// =============================================================================

#[cfg(test)]
mod phase_b_tests {
    use super::*;

    // Task B1: TcpConnection構造体テスト
    #[test]
    fn test_tcp_connection_new() {
        let remote_ip = Ipv4Addr::new(192, 168, 1, 100);
        let remote_port = 8080;

        let conn = TcpConnection::new(remote_ip, remote_port).unwrap();

        // 基本フィールドの確認
        assert_eq!(conn.state, TcpState::Closed);
        assert_eq!(conn.remote_ip, remote_ip);
        assert_eq!(conn.remote_port, remote_port);
        assert!(conn.socket_fd > 0); // 有効なソケットファイルディスクリプタ
        assert!(conn.local_port > 0); // 動的に割り当てられたポート
    }

    // Task B2: ISN生成テスト
    #[test]
    fn test_isn_generation_basic() {
        let isn1 = generate_isn();
        let isn2 = generate_isn();

        // 基本的な値チェック
        assert_ne!(isn1, 0, "ISN should not be zero");
        assert_ne!(isn2, 0, "ISN should not be zero");

        // 時間ベースなので通常は異なる値になる
        println!("Generated ISNs: {} and {}", isn1, isn2);
    }

    #[test]
    fn test_isn_generation_properties() {
        let mut isns = Vec::new();

        // 複数のISNを生成
        for _ in 0..5 {
            isns.push(generate_isn());
            std::thread::sleep(Duration::from_millis(1));
        }

        // すべて0以外
        for isn in &isns {
            assert_ne!(*isn, 0, "ISN should not be zero");
        }

        // 基本的な一意性チェック（時間ベースなので通常は異なる）
        let unique_count = {
            let mut sorted = isns.clone();
            sorted.sort();
            sorted.dedup();
            sorted.len()
        };

        println!("Generated {} ISNs, {} unique", isns.len(), unique_count);
    }

    // Task B4: パケット送信インフラの基本テスト
    #[test]
    fn test_socket_creation() {
        // TcpConnection::newでraw socketが作成されることをテスト
        let remote_ip = Ipv4Addr::new(127, 0, 0, 1);
        let conn = TcpConnection::new(remote_ip, 80).unwrap();

        // ソケットファイルディスクリプタが有効
        assert!(conn.socket_fd > 0);
    }
}

// =============================================================================
// Phase C: SYNパケット送信機能 - TDD Tests
// =============================================================================

#[cfg(test)]
mod phase_c_tests {
    use super::*;

    // Task C1: SYNパケット構築テスト
    #[test]
    fn test_syn_packet_creation() {
        // TcpConnection実装後に有効化
        let remote_ip = Ipv4Addr::new(127, 0, 0, 1);
        let conn = TcpConnection::new(remote_ip, 80).unwrap();

        let syn_packet = conn.create_syn_packet().unwrap();

        // TCPヘッダーのサイズチェック
        assert_eq!(syn_packet.len(), 20);

        // TCPヘッダーのフィールド確認

        // source port
        assert_eq!(
            u16::from_be_bytes([syn_packet[0], syn_packet[1]]),
            conn.local_port
        );
        // dest port
        assert_eq!(
            u16::from_be_bytes([syn_packet[2], syn_packet[3]]),
            conn.remote_port
        );
        // seq number
        assert_eq!(
            u32::from_be_bytes([syn_packet[4], syn_packet[5], syn_packet[6], syn_packet[7]]),
            conn.local_seq
        );
        // ack number
        assert_eq!(
            u32::from_be_bytes([syn_packet[8], syn_packet[9], syn_packet[10], syn_packet[11]]),
            0
        );

        // TCPフラグの確認
        let tcp_flags = syn_packet[13];
        assert_eq!(tcp_flags & 0x02, 0x02); // SYNフラグ
        assert_eq!(tcp_flags & 0x10, 0x00); // ACKフラグは未設定
    }

    // Task C2-C3: SYN送信機能テスト
    #[test]
    fn test_send_syn() {
        let remote_ip = Ipv4Addr::new(127, 0, 0, 1);
        let mut conn = TcpConnection::new(remote_ip, 80).unwrap();

        // 初期状態確認
        assert_eq!(conn.state, TcpState::Closed);
        let initial_seq = conn.local_seq;

        // SYN送信
        conn.send_syn().unwrap();

        // 状態変化確認
        assert_eq!(conn.state, TcpState::SynSent);
        assert_ne!(conn.local_seq, initial_seq); // ISNが設定された
        assert_ne!(conn.local_seq, 0);
    }
}

// =============================================================================
// Phase D: SYN-ACKパケット受信処理 - TDD Tests
// =============================================================================

#[cfg(test)]
mod phase_d_tests {
    use super::*;

    // Task D1: タイムアウト付き受信テスト
    #[test]
    fn test_receive_timeout() {
        let remote_ip = Ipv4Addr::new(192, 168, 255, 254); // 到達不可能
        let conn = TcpConnection::new(remote_ip, 12345).unwrap();

        let start = Instant::now();
        let result = conn.receive_packet_timeout(1); // 1秒でタイムアウト
        let elapsed = start.elapsed();

        // タイムアウトエラー
        assert!(result.is_err());
        assert!(elapsed >= Duration::from_secs(1));
        assert!(elapsed < Duration::from_secs(2)); // 多少のマージン
    }

    // Task D2: 受信パケット解析テスト
    #[test]
    fn test_packet_parsing() {
        let remote_ip = Ipv4Addr::new(127, 0, 0, 1);
        let conn = TcpConnection::new(remote_ip, 80).unwrap();

        // Step2のTcpHeaderで正しいTCPヘッダーを作成
        let tcp_header = TcpHeader::new(
            80,              // source port (remote)
            conn.local_port, // destination port (local)
            1000,            // sequence number
            2000,            // ack number
            tcp_flags::SYN,  // flags
            8192,            // window
        );

        // 完全なパケットを構築（IP + TCP）
        let ip_header = IpHeader::new(remote_ip, conn.local_ip, TCP_HEADER_SIZE as u16);
        let mut packet = ip_header.to_bytes();
        packet.extend_from_slice(&tcp_header.to_bytes());

        // パース実行
        let result = conn.parse_received_packet(&packet);

        // 結果検証
        assert!(result.is_ok(), "Packet parsing should succeed");

        let parsed_tcp = result.unwrap();
        assert_eq!(
            parsed_tcp.get_destination_port(),
            conn.local_port,
            "Destination port should match local port"
        );
    }

    // Task D3: SYN-ACK検証テスト
    #[test]
    fn test_syn_ack_validation() {
        let remote_ip = Ipv4Addr::new(127, 0, 0, 1);
        let mut conn = TcpConnection::new(remote_ip, 80).unwrap();
        conn.local_seq = 1000; // テスト用に設定

        // 正しいSYN-ACKパケット用のTcpHeader
        let syn_ack_header = TcpHeader::new(
            conn.remote_port,                // source port (remote)
            conn.local_port,                 // destination port (local)
            2000,                            // sequence number (サーバーのISN)
            1001,                            // ack number (local_seq + 1)
            tcp_flags::SYN | tcp_flags::ACK, // SYN + ACK flags
            8192,                            // window
        );

        assert!(conn.is_correct_syn_ack(&syn_ack_header));

        // 不正なフラグ（SYNのみ）
        let syn_only_header = TcpHeader::new(
            conn.remote_port,
            conn.local_port,
            2000,
            1001,
            tcp_flags::SYN, // SYN only (ACKなし)
            8192,
        );

        assert!(!conn.is_correct_syn_ack(&syn_only_header));

        // 不正なACK番号
        let wrong_ack_header = TcpHeader::new(
            conn.remote_port,
            conn.local_port,
            2000,
            1000, // wrong ack (should be local_seq + 1 = 1001)
            tcp_flags::SYN | tcp_flags::ACK,
            8192,
        );

        assert!(!conn.is_correct_syn_ack(&wrong_ack_header));

        // ポート番号違い
        let wrong_port_number = TcpHeader::new(
            9999, // wrong source port
            conn.local_port,
            2000,
            1001,
            tcp_flags::SYN | tcp_flags::ACK,
            8192,
        );

        assert!(!conn.is_correct_syn_ack(&wrong_port_number));
    }

    // Task D4: ACK番号チェック機能 - Task D3に統合済み
    // test_syn_ack_validation() で ACK番号の正確性チェックをテスト済み
}

// =============================================================================
// Phase E: ACKパケット送信と接続完了 - TDD Tests
// =============================================================================

#[cfg(test)]
mod phase_e_tests {
    use super::*;

    // Task E1: ACKパケット構築テスト
    #[test]
    fn test_ack_packet_creation() {
        // create_ack_packet実装後に有効化
        /*
        let remote_ip = Ipv4Addr::new(127, 0, 0, 1);
        let mut conn = TcpConnection::new(remote_ip, 80).unwrap();
        conn.local_seq = 1000;

        let ack_packet = conn.create_ack_packet(2000).unwrap();

        // パケット形式の基本チェック
        assert!(ack_packet.len() >= 40);

        // IPヘッダーの確認
        assert_eq!(ack_packet[0] >> 4, 4);
        assert_eq!(ack_packet[9], 6); // TCP

        // TCPヘッダーのACKフラグチェック
        let ip_header_len = ((ack_packet[0] & 0x0F) * 4) as usize;
        let tcp_flags = ack_packet[ip_header_len + 13];
        assert_eq!(tcp_flags & 0x10, 0x10); // ACKフラグ
        assert_eq!(tcp_flags & 0x02, 0x00); // SYNフラグは未設定
        */
    }

    // Task E2: ACK送信テスト
    #[test]
    fn test_ack_sending() {
        // send_ack実装後に有効化
        /*
        let remote_ip = Ipv4Addr::new(127, 0, 0, 1);
        let mut conn = TcpConnection::new(remote_ip, 80).unwrap();
        conn.local_seq = 1000;

        // ACK送信
        conn.send_ack(2001).unwrap();

        // リモートシーケンス番号が更新されている
        assert_eq!(conn.remote_seq, 2000); // ack_number - 1
        */
    }

    // Task E3: 接続確立完了テスト
    #[test]
    fn test_handshake_completion() {
        let remote_ip = Ipv4Addr::new(127, 0, 0, 1);
        let mut conn = TcpConnection::new(remote_ip, 80).expect("Failed to create connection");

        // SYN-SENT状態から開始（テスト用）
        conn.state = TcpState::SynSent;
        conn.local_seq = 1000;

        // 接続完了
        conn.complete_handshake();

        // 状態変化の確認
        assert_eq!(conn.state, TcpState::Established);
        assert_eq!(conn.local_seq, 1001); // SYN消費で+1
        assert!(conn.is_connected());
    }

    // Task E4: 接続確認テスト
    #[test]
    fn test_connection_status() {
        let remote_ip = Ipv4Addr::new(127, 0, 0, 1);
        let mut conn = TcpConnection::new(remote_ip, 80).expect("Failed to create connection");

        // 各状態での接続確認
        assert!(!conn.is_connected()); // CLOSED

        conn.state = TcpState::SynSent;
        assert!(!conn.is_connected()); // SYN-SENT

        conn.state = TcpState::Established;
        assert!(conn.is_connected()); // ESTABLISHED
    }
}

// =============================================================================
// Phase F: 統合テストと動作確認 - Integration Tests
// =============================================================================

#[cfg(test)]
mod phase_f_tests {
    use super::*;

    // Task F1: 完全handshakeテスト
    #[test]
    #[ignore] // ネットワーク接続が必要
    fn test_complete_handshake() {
        // connect実装後に有効化
        /*
        let remote_ip = Ipv4Addr::new(127, 0, 0, 1);
        let mut conn = TcpConnection::new(remote_ip, 8080).unwrap();

        // 3-way handshake実行
        let result = conn.connect(5);

        // 成功時の確認（netcatが起動していれば）
        if result.is_ok() {
            assert_eq!(conn.state, TcpState::Established);
            assert!(conn.is_connected());
            assert_ne!(conn.local_seq, 0);
            assert_ne!(conn.remote_seq, 0);
        }
        */
    }

    // Task F2: 実サーバーテスト
    #[test]
    #[ignore] // 外部ネットワーク接続が必要
    fn test_real_server_connection() {
        /*
        // httpbin.org など公開サーバーとのテスト
        let remote_ip = Ipv4Addr::new(54, 92, 72, 139); // httpbin.org example IP
        let mut conn = TcpConnection::new(remote_ip, 80).unwrap();

        let start = Instant::now();
        let result = conn.connect(10);
        let elapsed = start.elapsed();

        if result.is_ok() {
            assert!(conn.is_connected());
            assert!(elapsed < Duration::from_secs(5));
        }
        */
    }

    // Task F3: Wiresharkキャプチャ用テスト
    #[test]
    #[ignore] // 手動実行用
    fn test_wireshark_capture() {
        println!("🧪 Wireshark capture test");
        println!("手順:");
        println!("1. Wiresharkを起動してloopbackインターフェースを選択");
        println!("2. フィルター: tcp.port == 8080");
        println!("3. 別ターミナルでnetcatを起動: nc -l 8080");
        println!("4. このテストを実行: cargo test test_wireshark_capture -- --ignored");

        std::thread::sleep(Duration::from_secs(3)); // 準備時間

        /*
        let remote_ip = Ipv4Addr::new(127, 0, 0, 1);
        let mut conn = TcpConnection::new(remote_ip, 8080).unwrap();

        println!("Starting 3-way handshake...");
        match conn.connect(10) {
            Ok(_) => println!("✅ Connection successful - check Wireshark!"),
            Err(e) => println!("❌ Connection failed: {} - but packets should be visible", e),
        }
        */

        std::thread::sleep(Duration::from_secs(2)); // キャプチャ完了待ち
    }

    // Task F4: エラーケーステスト
    #[test]
    fn test_connection_timeout() {
        // 到達不可能なアドレスでタイムアウトテスト
        /*
        let remote_ip = Ipv4Addr::new(192, 168, 255, 254);
        let mut conn = TcpConnection::new(remote_ip, 12345).unwrap();

        let start = Instant::now();
        let result = conn.connect(2); // 2秒でタイムアウト
        let elapsed = start.elapsed();

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("timeout") || error_msg.contains("Timeout"));

        assert!(elapsed >= Duration::from_secs(2));
        assert!(elapsed < Duration::from_secs(4));
        assert_ne!(conn.state, TcpState::Established);
        */
    }

    #[test]
    fn test_connection_refused() {
        // 接続拒否テスト
        /*
        let remote_ip = Ipv4Addr::new(127, 0, 0, 1);
        let mut conn = TcpConnection::new(remote_ip, 65432).unwrap(); // 未使用ポート

        let result = conn.connect(2);

        if result.is_err() {
            let error = result.unwrap_err().to_string();
            assert!(error.contains("timeout") ||
                   error.contains("refused") ||
                   error.contains("Timeout"));
        }
        */
    }
}

// =============================================================================
// Performance Tests
// =============================================================================

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    #[ignore] // パフォーマンステストは手動実行
    fn test_connection_performance() {
        /*
        let remote_ip = Ipv4Addr::new(127, 0, 0, 1);
        let mut total_time = Duration::new(0, 0);
        let iterations = 5;

        for i in 0..iterations {
            let mut conn = TcpConnection::new(remote_ip, 8080).unwrap();

            let start = Instant::now();
            let _result = conn.connect(5);
            let elapsed = start.elapsed();

            total_time += elapsed;
            println!("Iteration {}: {:?}", i + 1, elapsed);
        }

        let average = total_time / iterations;
        println!("Average connection time: {:?}", average);

        assert!(average < Duration::from_secs(5));
        */
    }
}

// =============================================================================
// TDD実行ガイド
// =============================================================================

/*
TDD実行手順 (Step 3):

1. Phase A (理解・設計):
   ```
   cargo test phase_a_tests::test_tcp_state_enum_basic
   ```
   Red → Green: TcpState enumを実装

2. Phase B (基本構造):
   ```
   cargo test phase_b_tests::test_isn_generation_basic
   cargo test phase_b_tests::test_tcp_connection_new
   ```
   Red → Green: generate_isn()とTcpConnection::new()を実装

3. Phase C (SYN送信):
   ```
   cargo test phase_c_tests::test_syn_packet_creation
   cargo test phase_c_tests::test_syn_sending
   ```
   Red → Green: create_syn_packet()とsend_syn()を実装

4. Phase D (SYN-ACK受信):
   ```
   cargo test phase_d_tests::test_receive_timeout
   cargo test phase_d_tests::test_syn_ack_validation
   ```
   Red → Green: 受信・解析・検証ロジックを実装

5. Phase E (ACK送信・完了):
   ```
   cargo test phase_e_tests::test_ack_packet_creation
   cargo test phase_e_tests::test_handshake_completion
   ```
   Red → Green: ACK送信と接続完了を実装

6. Phase F (統合テスト):
   ```
   cargo test phase_f_tests --ignored
   ```
   実際のサーバーとの接続テスト

各フェーズでRed → Green → Refactorサイクルを実践。
コメントアウトされたテストは対応する実装完了後に有効化。
*/
