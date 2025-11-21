use super::*;

// =============================================================================
// Phase A: シーケンス番号の定義 - TDD Tests
// =============================================================================

#[cfg(test)]
mod phase_a_tests {
    use super::*;

    // Task A1: SequenceNumber構造体の基本テスト
    #[test]
    fn test_sequence_number_creation() {
        let seq = SequenceNumber::new(12345);
        assert_eq!(seq.value(), 12345);

        let seq_zero = SequenceNumber::new(0);
        assert_eq!(seq_zero.value(), 0);

        let seq_max = SequenceNumber::new(u32::MAX);
        assert_eq!(seq_max.value(), u32::MAX);
    }

    #[test]
    fn test_sequence_number_equality() {
        let seq1 = SequenceNumber::new(1000);
        let seq2 = SequenceNumber::new(1000);
        let seq3 = SequenceNumber::new(2000);

        assert_eq!(seq1, seq2);
        assert_ne!(seq1, seq3);
    }

    // Task A2: シーケンス番号の演算テスト
    #[test]
    fn test_wrapping_add() {
        let seq = SequenceNumber::new(100);
        let result = seq.wrapping_add(50);
        assert_eq!(result.value(), 150);

        // ラップアラウンドのテスト
        let seq_near_max = SequenceNumber::new(u32::MAX - 10);
        let wrapped = seq_near_max.wrapping_add(20);
        assert_eq!(wrapped.value(), 9); // (u32::MAX - 10) + 20 = 9 (wraps around)
    }

    #[test]
    fn test_wrapping_sub() {
        let seq1 = SequenceNumber::new(200);
        let seq2 = SequenceNumber::new(100);
        let diff = seq1.wrapping_sub(seq2);
        assert_eq!(diff, 100);

        // ラップアラウンドのテスト
        let seq_low = SequenceNumber::new(10);
        let seq_high = SequenceNumber::new(u32::MAX - 10);
        let wrapped_diff = seq_low.wrapping_sub(seq_high);
        assert_eq!(wrapped_diff, 21); // 10 - (u32::MAX - 10) wraps
    }

    // Task A3: シーケンス番号の比較テスト
    #[test]
    fn test_sequence_number_comparison() {
        let seq1 = SequenceNumber::new(1000);
        let seq2 = SequenceNumber::new(2000);
        let seq3 = SequenceNumber::new(1000);

        assert!(seq1 < seq2);
        assert!(seq2 > seq1);
        assert_eq!(seq1.cmp(&seq3), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_sequence_number_wraparound_comparison() {
        // ラップアラウンド時の比較
        let seq_near_max = SequenceNumber::new(u32::MAX - 100);
        let seq_wrapped = SequenceNumber::new(100);

        // seq_wrapped は seq_near_max より大きい（ラップアラウンド考慮）
        assert!(seq_wrapped > seq_near_max);
    }

    // Task A4: ISN生成テスト
    #[test]
    fn test_isn_generation() {
        let isn1 = generate_isn();
        let isn2 = generate_isn();

        // ISNは毎回異なる値を生成するべき
        // （時刻ベースなので、極めて短時間でない限り異なる）
        // 注意: このテストは確率的に失敗する可能性がある
        assert_ne!(isn1, isn2);
    }

    #[test]
    fn test_isn_is_valid() {
        let isn = generate_isn();
        // ISNは有効な32ビット値であること
        let _value = isn.value(); // パニックしないことを確認
    }
}

// =============================================================================
// Phase B: 送信バッファの実装 - TDD Tests
// =============================================================================

#[cfg(test)]
mod phase_b_tests {
    use super::*;

    // Task B1, B2: SendBufferの基本機能
    #[test]
    fn test_send_buffer_creation() {
        let seq = SequenceNumber::new(1000);
        let buffer = SendBuffer::new(seq);

        assert_eq!(buffer.next_seq(), seq);
        assert_eq!(buffer.unacked_seq(), seq);
        assert_eq!(buffer.available_space(), 65536); // 64KB
    }

    #[test]
    fn test_send_buffer_write() {
        let seq = SequenceNumber::new(1000);
        let mut buffer = SendBuffer::new(seq);

        let data = b"Hello, TCP!";
        let written = buffer.write(data).unwrap();

        assert_eq!(written, data.len());
        assert_eq!(buffer.available_space(), 65536 - data.len());
    }

    #[test]
    fn test_send_buffer_write_full() {
        let seq = SequenceNumber::new(1000);
        let mut buffer = SendBuffer::new(seq);

        // バッファを満杯にする
        let large_data = vec![0u8; 65536];
        let written = buffer.write(&large_data).unwrap();
        assert_eq!(written, 65536);
        assert_eq!(buffer.available_space(), 0);

        // さらに書き込もうとするとエラー
        let result = buffer.write(b"overflow");
        assert!(result.is_err());
    }

    // Task B3: peek と consume
    #[test]
    fn test_send_buffer_peek() {
        let seq = SequenceNumber::new(1000);
        let mut buffer = SendBuffer::new(seq);

        let data = b"Test data";
        buffer.write(data).unwrap();

        let peeked = buffer.peek(4);
        assert_eq!(peeked, b"Test");

        // peekは削除しないので、もう一度読める
        let peeked_again = buffer.peek(9);
        assert_eq!(peeked_again, b"Test data");
    }

    #[test]
    fn test_send_buffer_consume() {
        let seq = SequenceNumber::new(1000);
        let mut buffer = SendBuffer::new(seq);

        let data = b"0123456789";
        buffer.write(data).unwrap();

        buffer.consume(5);
        assert_eq!(buffer.next_seq(), SequenceNumber::new(1005));
    }

    // Task B4: acknowledge
    #[test]
    fn test_send_buffer_acknowledge() {
        let seq = SequenceNumber::new(1000);
        let mut buffer = SendBuffer::new(seq);

        let data = b"0123456789";
        buffer.write(data).unwrap();
        buffer.consume(10); // 全データ送信済み

        let ack_seq = SequenceNumber::new(1005);
        let acked_bytes = buffer.acknowledge(ack_seq).unwrap();

        assert_eq!(acked_bytes, 5);
        assert_eq!(buffer.unacked_seq(), ack_seq);
        assert!(buffer.available_space() > 65536 - 10); // バッファから削除された
    }

    #[test]
    fn test_send_buffer_unacked_data() {
        let seq = SequenceNumber::new(1000);
        let mut buffer = SendBuffer::new(seq);

        let data = b"0123456789";
        buffer.write(data).unwrap();

        assert_eq!(buffer.unacked_data(), 0); // まだ送信していない

        buffer.consume(10);
        assert_eq!(buffer.unacked_data(), 10); // 送信済み未確認

        buffer.acknowledge(SequenceNumber::new(1005)).unwrap();
        assert_eq!(buffer.unacked_data(), 5); // 5バイト確認済み
    }
}

// =============================================================================
// Phase C: 受信バッファの実装 - TDD Tests
// =============================================================================

#[cfg(test)]
mod phase_c_tests {
    use super::*;

    // Task C1, C2: ReceiveBufferの基本機能
    #[test]
    fn test_receive_buffer_creation() {
        let seq = SequenceNumber::new(2000);
        let buffer = ReceiveBuffer::new(seq);

        assert_eq!(buffer.next_expected(), seq);
        assert_eq!(buffer.available(), 0);
        assert!(!buffer.has_gap());
    }

    #[test]
    fn test_receive_buffer_in_order() {
        let seq = SequenceNumber::new(2000);
        let mut buffer = ReceiveBuffer::new(seq);

        let data = b"Hello";
        buffer.receive(seq, data).unwrap();

        assert_eq!(buffer.available(), data.len());
        assert_eq!(buffer.next_expected(), seq.wrapping_add(data.len() as u32));
    }

    #[test]
    fn test_receive_buffer_duplicate() {
        let seq = SequenceNumber::new(2000);
        let mut buffer = ReceiveBuffer::new(seq);

        let data = b"Hello";
        buffer.receive(seq, data).unwrap();

        // 重複データを受信（無視されるべき）
        buffer.receive(seq, data).unwrap();
        assert_eq!(buffer.available(), data.len()); // 増えない
    }

    // Task C3: read
    #[test]
    fn test_receive_buffer_read() {
        let seq = SequenceNumber::new(2000);
        let mut buffer = ReceiveBuffer::new(seq);

        let data = b"Hello, World!";
        buffer.receive(seq, data).unwrap();

        let read_data = buffer.read(5);
        assert_eq!(read_data, b"Hello");
        assert_eq!(buffer.available(), data.len() - 5);

        let remaining = buffer.read(100);
        assert_eq!(remaining, b", World!");
        assert_eq!(buffer.available(), 0);
    }

    // Task C4: 順序外データの処理
    #[test]
    fn test_receive_buffer_out_of_order() {
        let seq = SequenceNumber::new(2000);
        let mut buffer = ReceiveBuffer::new(seq);

        // セグメント2を先に受信（順序外）
        let data2 = b"World";
        buffer.receive(seq.wrapping_add(5), data2).unwrap();

        assert_eq!(buffer.available(), 0); // まだ読めない
        assert!(buffer.has_gap()); // ギャップあり

        // セグメント1を受信（穴が埋まる）
        let data1 = b"Hello";
        buffer.receive(seq, data1).unwrap();

        assert_eq!(buffer.available(), 10); // 両方読めるようになった
        assert!(!buffer.has_gap()); // ギャップなし

        let all_data = buffer.read(100);
        assert_eq!(all_data, b"HelloWorld");
    }

    #[test]
    fn test_receive_buffer_multiple_gaps() {
        let seq = SequenceNumber::new(2000);
        let mut buffer = ReceiveBuffer::new(seq);

        // セグメント3, 1を受信（セグメント2が欠落）
        buffer.receive(seq.wrapping_add(10), b"!").unwrap();
        buffer.receive(seq, b"Hello").unwrap();

        assert_eq!(buffer.available(), 5); // セグメント1のみ
        assert!(buffer.has_gap());

        // セグメント2を受信
        buffer.receive(seq.wrapping_add(5), b"World").unwrap();

        assert_eq!(buffer.available(), 11); // 全部読める
        assert!(!buffer.has_gap());
    }
}

// =============================================================================
// Phase D: セグメント化ロジック - TDD Tests
// =============================================================================

#[cfg(test)]
mod phase_d_tests {
    use super::*;

    // Task D1: Segment構造
    #[test]
    fn test_segment_creation() {
        let seq = SequenceNumber::new(3000);
        let data = vec![1, 2, 3, 4, 5];
        let segment = Segment::new(seq, data.clone());

        assert_eq!(segment.seq, seq);
        assert_eq!(segment.data, data);
        assert_eq!(segment.len(), 5);
    }

    // Task D2: セグメント化
    #[test]
    fn test_segment_data_single() {
        let data = b"Hello";
        let seq = SequenceNumber::new(4000);

        let segments = segment_data(data, DEFAULT_MSS, seq);

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].seq, seq);
        assert_eq!(segments[0].data, data);
    }

    #[test]
    fn test_segment_data_multiple() {
        let data = vec![0u8; DEFAULT_MSS * 2 + 100]; // 2.5セグメント分
        let seq = SequenceNumber::new(5000);

        let segments = segment_data(&data, DEFAULT_MSS, seq);

        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].len(), DEFAULT_MSS);
        assert_eq!(segments[1].len(), DEFAULT_MSS);
        assert_eq!(segments[2].len(), 100);

        // シーケンス番号が連続しているか確認
        assert_eq!(segments[0].seq, seq);
        assert_eq!(segments[1].seq, seq.wrapping_add(DEFAULT_MSS as u32));
        assert_eq!(segments[2].seq, seq.wrapping_add((DEFAULT_MSS * 2) as u32));
    }

    #[test]
    fn test_segment_data_empty() {
        let data = b"";
        let seq = SequenceNumber::new(6000);

        let segments = segment_data(data, DEFAULT_MSS, seq);

        // 空データは空のベクタまたは空セグメントを返す実装もあり得る
        // ここでは空ベクタを期待
        assert!(segments.is_empty() || (segments.len() == 1 && segments[0].is_empty()));
    }

    // Task D3: セグメント検証
    #[test]
    fn test_validate_segment_within_window() {
        let seq = SequenceNumber::new(7000);
        let data = vec![1, 2, 3, 4, 5];
        let segment = Segment::new(seq, data);

        let expected_seq = SequenceNumber::new(7000);
        let window_size = 1000;

        assert!(validate_segment(&segment, expected_seq, window_size));
    }

    #[test]
    fn test_validate_segment_outside_window() {
        let seq = SequenceNumber::new(9000); // ウィンドウ外
        let data = vec![1, 2, 3, 4, 5];
        let segment = Segment::new(seq, data);

        let expected_seq = SequenceNumber::new(7000);
        let window_size = 1000; // 7000-8000の範囲

        assert!(!validate_segment(&segment, expected_seq, window_size));
    }

    #[test]
    fn test_validate_segment_at_boundary() {
        let seq = SequenceNumber::new(7999); // ウィンドウ境界
        let data = vec![1, 2, 3];
        let segment = Segment::new(seq, data);

        let expected_seq = SequenceNumber::new(7000);
        let window_size = 1000;

        assert!(validate_segment(&segment, expected_seq, window_size));
    }
}

// =============================================================================
// Phase E: データ送受信統合 - TDD Tests
// =============================================================================

#[cfg(test)]
mod phase_e_tests {
    use super::*;

    // Task E1: TcpConnection作成
    #[test]
    fn test_tcp_connection_creation() {
        let local_isn = SequenceNumber::new(1000);
        let remote_isn = SequenceNumber::new(2000);

        let conn = TcpConnection::new(local_isn, remote_isn);

        assert_eq!(conn.send_buffer().next_seq(), local_isn);
        assert_eq!(conn.recv_buffer().next_expected(), remote_isn);
    }

    // Task E2: 送信処理
    #[test]
    fn test_tcp_connection_send() {
        let local_isn = SequenceNumber::new(1000);
        let remote_isn = SequenceNumber::new(2000);
        let mut conn = TcpConnection::new(local_isn, remote_isn);

        let data = b"Hello, TCP!";
        let sent = conn.send(data).unwrap();

        assert_eq!(sent, data.len());
    }

    // Task E3: 受信処理
    #[test]
    fn test_tcp_connection_receive() {
        let local_isn = SequenceNumber::new(1000);
        let remote_isn = SequenceNumber::new(2000);
        let mut conn = TcpConnection::new(local_isn, remote_isn);

        let data = b"Received data";
        conn.receive(remote_isn, data).unwrap();

        assert_eq!(conn.recv_buffer().available(), data.len());
    }

    #[test]
    fn test_tcp_connection_read() {
        let local_isn = SequenceNumber::new(1000);
        let remote_isn = SequenceNumber::new(2000);
        let mut conn = TcpConnection::new(local_isn, remote_isn);

        let data = b"Test data";
        conn.receive(remote_isn, data).unwrap();

        let read_data = conn.read(100);
        assert_eq!(read_data, data);
    }

    // Task E4: ACK処理
    #[test]
    fn test_tcp_connection_generate_ack() {
        let local_isn = SequenceNumber::new(1000);
        let remote_isn = SequenceNumber::new(2000);
        let mut conn = TcpConnection::new(local_isn, remote_isn);

        let data = b"Data";
        conn.receive(remote_isn, data).unwrap();

        let ack = conn.generate_ack();
        assert_eq!(ack, remote_isn.wrapping_add(data.len() as u32));
    }

    #[test]
    fn test_tcp_connection_process_ack() {
        let local_isn = SequenceNumber::new(1000);
        let remote_isn = SequenceNumber::new(2000);
        let mut conn = TcpConnection::new(local_isn, remote_isn);

        let data = b"Sending data";
        conn.send(data).unwrap();

        // データを送信したことにする（実際のネットワーク送信は省略）
        // ここではバッファ内部のconsume操作を想定
        // テストでは直接バッファ操作するか、別途メソッドを用意

        let ack_seq = local_isn.wrapping_add(data.len() as u32);
        conn.process_ack(ack_seq).unwrap();
    }
}

// =============================================================================
// Phase F: 統合テストと検証 - TDD Tests
// =============================================================================

#[cfg(test)]
mod phase_f_tests {
    use super::*;

    // Task F1: 完全な送受信フロー
    #[test]
    fn test_complete_send_receive_flow() {
        let local_isn = SequenceNumber::new(1000);
        let remote_isn = SequenceNumber::new(2000);

        let mut sender = TcpConnection::new(local_isn, remote_isn);
        let mut receiver = TcpConnection::new(remote_isn, local_isn);

        // 送信側: データを送信
        let send_data = b"Hello from sender";
        sender.send(send_data).unwrap();

        // （実際のネットワーク送信は省略、直接受信側に渡す想定）
        receiver.receive(local_isn, send_data).unwrap();

        // 受信側: データを読み取る
        let recv_data = receiver.read(100);
        assert_eq!(recv_data, send_data);

        // 受信側: ACKを生成
        let ack = receiver.generate_ack();
        assert_eq!(ack, local_isn.wrapping_add(send_data.len() as u32));

        // 送信側: ACKを処理
        // （実際には送信バッファのconsumeが必要だが、ここでは簡略化）
    }

    // Task F2: セグメント化テスト
    #[test]
    fn test_large_data_segmentation() {
        let data = vec![0xAB; DEFAULT_MSS * 3]; // 3セグメント分
        let seq = SequenceNumber::new(10000);

        let segments = segment_data(&data, DEFAULT_MSS, seq);

        assert_eq!(segments.len(), 3);
        for (i, segment) in segments.iter().enumerate() {
            assert_eq!(segment.len(), DEFAULT_MSS);
            assert_eq!(segment.seq, seq.wrapping_add((i * DEFAULT_MSS) as u32));
        }
    }

    // Task F3: 順序外受信テスト
    #[test]
    fn test_out_of_order_complete_scenario() {
        let local_isn = SequenceNumber::new(1000);
        let remote_isn = SequenceNumber::new(2000);
        let mut conn = TcpConnection::new(local_isn, remote_isn);

        // セグメント3, 1, 2の順で受信
        let seg1 = b"AAA";
        let seg2 = b"BBB";
        let seg3 = b"CCC";

        conn.receive(remote_isn.wrapping_add(6), seg3).unwrap();
        assert_eq!(conn.recv_buffer().available(), 0); // まだ読めない

        conn.receive(remote_isn, seg1).unwrap();
        assert_eq!(conn.recv_buffer().available(), 3); // seg1のみ

        conn.receive(remote_isn.wrapping_add(3), seg2).unwrap();
        assert_eq!(conn.recv_buffer().available(), 9); // 全部読める

        let all_data = conn.read(100);
        assert_eq!(all_data, b"AAABBBBCCC");
    }

    // Task F4: バッファ境界テスト
    #[test]
    fn test_buffer_limits() {
        let local_isn = SequenceNumber::new(1000);
        let remote_isn = SequenceNumber::new(2000);
        let mut conn = TcpConnection::new(local_isn, remote_isn);

        // 送信バッファを満杯にする
        let max_data = vec![0u8; 65536];
        conn.send(&max_data).unwrap();

        // さらに書き込もうとするとエラー
        let result = conn.send(b"overflow");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_buffer_read() {
        let local_isn = SequenceNumber::new(1000);
        let remote_isn = SequenceNumber::new(2000);
        let mut conn = TcpConnection::new(local_isn, remote_isn);

        // 空のバッファから読み取り
        let data = conn.read(100);
        assert!(data.is_empty());
    }

    // RFC準拠の確認
    #[test]
    fn test_sequence_number_wraparound_handling() {
        // シーケンス番号のラップアラウンドを含むシナリオ
        let seq_near_max = SequenceNumber::new(u32::MAX - 100);
        let data = vec![0u8; 200];

        let segments = segment_data(&data, 150, seq_near_max);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].seq, seq_near_max);
        assert_eq!(segments[1].seq, seq_near_max.wrapping_add(150));

        // 2番目のセグメントはラップアラウンドしている
        assert!(segments[1].seq.value() < 100);
    }
}

/*
TDD実行手順:
1. cargo test phase_a_tests で Phase A のテストを実行
2. Red → Green → Refactor サイクルを回す
3. 各タスクを順に実装してテストをパスさせる
4. 次のフェーズに進む

推奨実行コマンド:
- cargo test phase_a_tests  -- Phase A のテスト
- cargo test phase_b_tests  -- Phase B のテスト
- cargo test phase_c_tests  -- Phase C のテスト
- cargo test phase_d_tests  -- Phase D のテスト
- cargo test phase_e_tests  -- Phase E のテスト
- cargo test phase_f_tests  -- Phase F のテスト
- cargo test --bin step05   -- すべてのテスト
*/
