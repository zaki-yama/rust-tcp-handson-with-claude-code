use super::*;

// =============================================================================
// Phase B: TCPヘッダー構造体の実装 - TDD Tests
// =============================================================================

#[cfg(test)]
mod phase_b_tests {
    use super::*;

    // Task B1 & B2: Constructor Test
    // 期待される動作: new()メソッドで正しくTCPヘッダーを作成
    #[test]
    fn test_tcp_header_creation_basic() {
        // Red: 最初は失敗する
        let header = TcpHeader::new(80, 12345, 1000, 0, tcp_flags::SYN, 8192);

        // 作成されたヘッダーのフィールドが正しく設定されているかチェック
        // Note: ネットワークバイトオーダーで格納されているので、比較時は変換が必要
        assert_eq!(u16::from_be(header.source_port), 80);
        assert_eq!(u16::from_be(header.destination_port), 12345);
        assert_eq!(u32::from_be(header.sequence_number), 1000);
        assert_eq!(u32::from_be(header.acknowledgment_number), 0);
        assert_eq!(u16::from_be(header.window_size), 8192);
        assert_eq!(header.checksum, 0); // 初期値は0
        assert_eq!(header.urgent_pointer, 0); // 初期値は0
    }

    #[test]
    fn test_tcp_header_data_offset_and_flags() {
        let header = TcpHeader::new(80, 443, 1000, 2000, tcp_flags::SYN | tcp_flags::ACK, 1024);

        // Data Offsetは5 (20バイト ÷ 4) であるべき
        assert_eq!(header.get_data_offset(), 5);

        // フラグが正しく設定されているかチェック
        let flags = header.get_flags();
        assert_eq!(flags, tcp_flags::SYN | tcp_flags::ACK);
        assert!(flags & tcp_flags::SYN != 0);
        assert!(flags & tcp_flags::ACK != 0);
        assert!(flags & tcp_flags::FIN == 0);
    }

    #[test]
    fn test_tcp_header_different_flags() {
        // 異なるフラグでのテスト
        let test_cases = [
            (tcp_flags::SYN, "SYN only"),
            (tcp_flags::ACK, "ACK only"),
            (tcp_flags::FIN, "FIN only"),
            (tcp_flags::RST, "RST only"),
            (tcp_flags::SYN | tcp_flags::ACK, "SYN+ACK"),
            (tcp_flags::FIN | tcp_flags::ACK, "FIN+ACK"),
        ];

        for (flags, description) in test_cases.iter() {
            let header = TcpHeader::new(8080, 3000, 500, 1500, *flags, 4096);
            assert_eq!(header.get_flags(), *flags, "Failed for {}", description);
        }
    }

    // Task B3: to_bytes() Test
    // 期待される動作: TCPヘッダーが正しく20バイトのバイト配列に変換される
    #[test]
    fn test_tcp_header_to_bytes_length() {
        let header = TcpHeader::new(80, 12345, 1000, 0, tcp_flags::SYN, 8192);
        let bytes = header.to_bytes();

        // TCPヘッダーは必ず20バイト
        assert_eq!(bytes.len(), TCP_HEADER_SIZE);
    }

    #[test]
    fn test_tcp_header_to_bytes_content() {
        // 既知の値でテスト
        let header = TcpHeader::new(80, 12345, 1000, 2000, tcp_flags::SYN, 8192);
        let bytes = header.to_bytes();

        // 最初の4バイト: source_port(80) + destination_port(12345)
        // 80 = 0x0050, 12345 = 0x3039 (network byte order)
        assert_eq!(bytes[0], 0x00); // source_port high byte
        assert_eq!(bytes[1], 0x50); // source_port low byte
        assert_eq!(bytes[2], 0x30); // destination_port high byte
        assert_eq!(bytes[3], 0x39); // destination_port low byte

        // 次の4バイト: sequence_number(1000)
        // 1000 = 0x000003E8 (network byte order)
        assert_eq!(bytes[4], 0x00); // seq high byte
        assert_eq!(bytes[5], 0x00);
        assert_eq!(bytes[6], 0x03);
        assert_eq!(bytes[7], 0xE8); // seq low byte

        // 次の4バイト: acknowledgment_number(2000)
        // 2000 = 0x000007D0 (network byte order)
        assert_eq!(bytes[8], 0x00); // ack high byte
        assert_eq!(bytes[9], 0x00);
        assert_eq!(bytes[10], 0x07);
        assert_eq!(bytes[11], 0xD0); // ack low byte
    }

    // Task B4: Round-trip Test (作成 → バイト変換 → パース)
    // これはPhase Cのparse()実装後に動作するが、ゴールイメージとして定義
    #[test]
    #[ignore] // Phase Cまで無視
    fn test_tcp_header_round_trip() {
        let original = TcpHeader::new(443, 8080, 12345, 67890, tcp_flags::ACK, 16384);
        let bytes = original.to_bytes();
        let parsed = TcpHeader::parse(&bytes).expect("Should parse successfully");

        // ラウンドトリップで値が保持されているかチェック
        assert_eq!(
            u16::from_be(original.source_port),
            u16::from_be(parsed.source_port)
        );
        assert_eq!(
            u16::from_be(original.destination_port),
            u16::from_be(parsed.destination_port)
        );
        assert_eq!(
            u32::from_be(original.sequence_number),
            u32::from_be(parsed.sequence_number)
        );
        assert_eq!(
            u32::from_be(original.acknowledgment_number),
            u32::from_be(parsed.acknowledgment_number)
        );
        assert_eq!(original.get_flags(), parsed.get_flags());
        assert_eq!(
            u16::from_be(original.window_size),
            u16::from_be(parsed.window_size)
        );
    }
}

// =============================================================================
// Phase C: パース機能の実装 - TDD Tests
// =============================================================================

#[cfg(test)]
mod phase_c_tests {
    use super::*;

    // Task C1: Basic Parse Test
    #[test]
    #[ignore] // Phase Cで実装時に有効化
    fn test_parse_valid_tcp_header() {
        // 手作りのTCPヘッダーバイト配列
        let tcp_bytes = [
            0x00, 0x50, // source_port = 80
            0x30, 0x39, // destination_port = 12345
            0x00, 0x00, 0x03, 0xE8, // sequence_number = 1000
            0x00, 0x00, 0x07, 0xD0, // acknowledgment_number = 2000
            0x50, 0x02, // data_offset=5, flags=SYN
            0x20, 0x00, // window_size = 8192
            0x00, 0x00, // checksum = 0
            0x00, 0x00, // urgent_pointer = 0
        ];

        let header = TcpHeader::parse(&tcp_bytes).expect("Should parse successfully");

        assert_eq!(u16::from_be(header.source_port), 80);
        assert_eq!(u16::from_be(header.destination_port), 12345);
        assert_eq!(u32::from_be(header.sequence_number), 1000);
        assert_eq!(u32::from_be(header.acknowledgment_number), 2000);
        assert_eq!(header.get_flags(), tcp_flags::SYN);
        assert_eq!(u16::from_be(header.window_size), 8192);
    }

    #[test]
    #[ignore] // Phase Cで実装時に有効化
    fn test_parse_invalid_length() {
        // 20バイト未満のデータ
        let short_data = [0x00, 0x50, 0x30, 0x39]; // 4バイトのみ

        let result = TcpHeader::parse(&short_data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Data too short for TCP header");
    }

    #[test]
    #[ignore] // Phase Cで実装時に有効化
    fn test_parse_different_flags() {
        let test_cases = [
            (tcp_flags::SYN, 0x50, 0x02),
            (tcp_flags::ACK, 0x50, 0x10),
            (tcp_flags::SYN | tcp_flags::ACK, 0x50, 0x12),
            (tcp_flags::FIN | tcp_flags::ACK, 0x50, 0x11),
        ];

        for (expected_flags, offset_byte, flags_byte) in test_cases.iter() {
            let mut tcp_bytes = [0u8; 20];
            tcp_bytes[12] = *offset_byte; // data_offset = 5
            tcp_bytes[13] = *flags_byte; // flags

            let header = TcpHeader::parse(&tcp_bytes).expect("Should parse");
            assert_eq!(header.get_flags(), *expected_flags);
        }
    }
}

// =============================================================================
// Phase D: チェックサム計算 - TDD Tests
// =============================================================================

#[cfg(test)]
mod phase_d_tests {
    use super::*;

    #[test]
    #[ignore] // Phase Dで実装時に有効化
    fn test_pseudo_header_creation() {
        // 192.168.1.1 -> 192.168.1.2, TCP length = 20
        let src_ip = u32::from_be_bytes([192, 168, 1, 1]);
        let dst_ip = u32::from_be_bytes([192, 168, 1, 2]);
        let tcp_length = 20;

        let pseudo_header = create_pseudo_header(src_ip, dst_ip, tcp_length);

        assert_eq!(pseudo_header.len(), 12);

        // Source IP (4 bytes)
        assert_eq!(&pseudo_header[0..4], &[192, 168, 1, 1]);
        // Destination IP (4 bytes)
        assert_eq!(&pseudo_header[4..8], &[192, 168, 1, 2]);
        // Zero + Protocol + TCP Length (4 bytes)
        assert_eq!(pseudo_header[8], 0); // zero
        assert_eq!(pseudo_header[9], 6); // TCP protocol
        assert_eq!(pseudo_header[10], 0); // TCP length high byte
        assert_eq!(pseudo_header[11], 20); // TCP length low byte
    }

    #[test]
    #[ignore] // Phase Dで実装時に有効化
    fn test_checksum_calculation() {
        let src_ip = u32::from_be_bytes([10, 0, 0, 1]);
        let dst_ip = u32::from_be_bytes([10, 0, 0, 2]);

        let mut header = TcpHeader::new(80, 8080, 1000, 2000, tcp_flags::SYN, 1024);
        let tcp_data = b""; // データなし

        header.calculate_checksum(src_ip, dst_ip, tcp_data);

        // チェックサムが計算されて0以外になっているかチェック
        assert_ne!(header.checksum, 0);

        // 検証テスト
        assert!(header.verify_checksum(src_ip, dst_ip, tcp_data));
    }

    #[test]
    #[ignore] // Phase Dで実装時に有効化
    fn test_checksum_with_data() {
        let src_ip = u32::from_be_bytes([127, 0, 0, 1]);
        let dst_ip = u32::from_be_bytes([127, 0, 0, 1]);

        let mut header = TcpHeader::new(12345, 80, 5000, 10000, tcp_flags::ACK, 2048);
        let tcp_data = b"Hello, TCP!";

        header.calculate_checksum(src_ip, dst_ip, tcp_data);

        // データありでも正しく検証できるかチェック
        assert!(header.verify_checksum(src_ip, dst_ip, tcp_data));

        // 異なるデータでは検証に失敗するかチェック
        let wrong_data = b"Wrong data!";
        assert!(!header.verify_checksum(src_ip, dst_ip, wrong_data));
    }
}

// =============================================================================
// Integration Tests - 統合テスト
// =============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    #[ignore] // 全Phase完了後に有効化
    fn test_complete_tcp_header_workflow() {
        let src_ip = u32::from_be_bytes([192, 168, 1, 100]);
        let dst_ip = u32::from_be_bytes([192, 168, 1, 200]);

        // 1. ヘッダー作成
        let mut original = TcpHeader::new(
            443,
            8080,
            123456,
            789012,
            tcp_flags::SYN | tcp_flags::ACK,
            4096,
        );

        // 2. チェックサム計算
        let tcp_data = b"Integration test data";
        original.calculate_checksum(src_ip, dst_ip, tcp_data);

        // 3. バイト配列変換
        let bytes = original.to_bytes();
        assert_eq!(bytes.len(), 20);

        // 4. パース
        let parsed = TcpHeader::parse(&bytes).expect("Should parse");

        // 5. チェックサム検証
        assert!(parsed.verify_checksum(src_ip, dst_ip, tcp_data));

        // 6. 値の整合性確認
        assert_eq!(
            u16::from_be(original.source_port),
            u16::from_be(parsed.source_port)
        );
        assert_eq!(
            u16::from_be(original.destination_port),
            u16::from_be(parsed.destination_port)
        );
        assert_eq!(original.get_flags(), parsed.get_flags());
    }

    #[test]
    #[ignore] // 全Phase完了後に有効化
    fn test_known_tcp_packet() {
        // Wiresharkでキャプチャした実際のTCPパケットでテスト
        // (実際の使用時は本物のパケットデータを使用)
        let real_tcp_packet = [
            // 実際のTCPパケットのバイト配列をここに配置
            0x04, 0xd2, 0x00, 0x50, // sport=1234, dport=80
            0x12, 0x34, 0x56, 0x78, // seq
            0x9a, 0xbc, 0xde, 0xf0, // ack
            0x50, 0x18, 0x01, 0x00, // offset+flags, window
            0xa1, 0xb2, 0x00, 0x00, // checksum, urgent
        ];

        match TcpHeader::parse(&real_tcp_packet) {
            Ok(header) => {
                println!("Real packet parsed successfully: {:?}", header);
                // 実際のパケットの値と照合
            }
            Err(e) => {
                println!("Failed to parse real packet: {}", e);
            }
        }
    }
}

// =============================================================================
// Test Utilities
// =============================================================================

#[cfg(test)]
mod test_utils {
    use super::*;

    /// テスト用のサンプルTCPヘッダー作成
    pub fn create_sample_header() -> TcpHeader {
        TcpHeader::new(80, 12345, 1000, 2000, tcp_flags::SYN, 8192)
    }

    /// テスト用のサンプルIPアドレス
    pub fn sample_ip_pair() -> (u32, u32) {
        let src = u32::from_be_bytes([192, 168, 1, 1]);
        let dst = u32::from_be_bytes([192, 168, 1, 2]);
        (src, dst)
    }
}

// =============================================================================
// TDD実行ガイド
// =============================================================================

/*
TDD実行手順:

1. Phase B実装:
   ```
   cargo test phase_b_tests::test_tcp_header_creation_basic
   ```
   Red → Green → Refactor のサイクルで Task B1, B2を実装

2. Phase B継続:
   ```
   cargo test phase_b_tests::test_tcp_header_to_bytes_length
   cargo test phase_b_tests::test_tcp_header_to_bytes_content
   ```
   Task B3を実装

3. Phase C実装 (ignoreを外して):
   ```
   cargo test phase_c_tests --ignored
   ```
   Task C1-C4を実装

4. Phase D実装 (ignoreを外して):
   ```
   cargo test phase_d_tests --ignored
   ```
   Task D1-D4を実装

5. 統合テスト:
   ```
   cargo test integration_tests --ignored
   ```

各テストが Green になったら次のPhaseに進む。
Red → Green → Refactor を繰り返して高品質なコードを作成する。
*/
