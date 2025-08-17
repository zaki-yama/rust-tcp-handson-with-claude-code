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
        // Note: packedフィールドの安全なアクセス
        let source_port = header.source_port;
        let destination_port = header.destination_port;
        let sequence_number = header.sequence_number;
        let acknowledgment_number = header.acknowledgment_number;
        let window_size = header.window_size;
        let checksum = header.checksum;
        let urgent_pointer = header.urgent_pointer;

        assert_eq!(source_port, 80);
        assert_eq!(destination_port, 12345);
        assert_eq!(sequence_number, 1000);
        assert_eq!(acknowledgment_number, 0);
        assert_eq!(window_size, 8192);
        assert_eq!(checksum, 0); // 初期値は0
        assert_eq!(urgent_pointer, 0); // 初期値は0
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
    // これはPhase Cのfrom_bytes()実装後に動作するが、ゴールイメージとして定義
    #[test]
    fn test_tcp_header_round_trip() {
        let original = TcpHeader::new(443, 8080, 12345, 67890, tcp_flags::ACK, 16384);
        let bytes = original.to_bytes();
        let parsed = TcpHeader::from_bytes(&bytes).expect("Should convert from bytes successfully");

        // ラウンドトリップで値が保持されているかチェック
        let source_port = original.source_port;
        let destination_port = original.destination_port;
        let sequence_number = original.sequence_number;
        let acknowledgment_number = original.acknowledgment_number;
        let window_size = original.window_size;
        // let checksum = original.checksum;
        // let urgent_pointer = original.urgent_pointer;

        let parsed_source_port = parsed.source_port;
        let parsed_destination_port = parsed.destination_port;
        let parsed_sequence_number = parsed.sequence_number;
        let parsed_acknowledgment_number = parsed.acknowledgment_number;
        let parsed_window_size = parsed.window_size;

        assert_eq!(source_port, parsed_source_port);
        assert_eq!(destination_port, parsed_destination_port);
        assert_eq!(sequence_number, parsed_sequence_number);
        assert_eq!(acknowledgment_number, parsed_acknowledgment_number);
        assert_eq!(original.get_flags(), parsed.get_flags());
        assert_eq!(window_size, parsed_window_size);
    }
}

// =============================================================================
// Phase C: from_bytes機能の実装 - TDD Tests
// =============================================================================

#[cfg(test)]
mod phase_c_tests {
    use super::*;

    // Task C1: Basic from_bytes Test
    #[test]
    fn test_from_bytes_valid_tcp_header() {
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

        let header =
            TcpHeader::from_bytes(&tcp_bytes).expect("Should convert from bytes successfully");

        // 全フィールドの正確性を検証（packed構造体対応）
        let source_port = header.source_port;
        let destination_port = header.destination_port;
        let sequence_number = header.sequence_number;
        let acknowledgment_number = header.acknowledgment_number;
        let window_size = header.window_size;
        let checksum = header.checksum;
        let urgent_pointer = header.urgent_pointer;

        assert_eq!(source_port, 80);
        assert_eq!(destination_port, 12345);
        assert_eq!(sequence_number, 1000);
        assert_eq!(acknowledgment_number, 2000);
        assert_eq!(header.get_flags(), tcp_flags::SYN);
        assert_eq!(window_size, 8192);
        assert_eq!(checksum, 0);
        assert_eq!(urgent_pointer, 0);
        assert_eq!(header.get_data_offset(), 5); // 20バイト ÷ 4
    }

    #[test]
    fn test_from_bytes_invalid_length() {
        // 20バイト未満のデータ
        let short_data = [0x00, 0x50, 0x30, 0x39]; // 4バイトのみ

        let result = TcpHeader::from_bytes(&short_data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Data too short for TCP header");
    }

    #[test]
    fn test_from_bytes_different_flags() {
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

            let header = TcpHeader::from_bytes(&tcp_bytes).expect("Should convert from bytes");
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
    fn test_rfc1071_algorithm_simple() {
        // RFC 1071の例: [0x45, 0x00, 0x00, 0x3c] のチェックサム
        let data = [0x45, 0x00, 0x00, 0x3c];
        let result = calculate_checksum_rfc1071(&data);

        // 手動計算: 0x4500 + 0x003c = 0x453c
        // 1の補数: !0x453c = 0xbac3
        let expected = 0xbac3;
        assert_eq!(result, expected, "RFC 1071 algorithm test failed");
    }

    #[test]
    fn test_rfc1071_algorithm_with_carry() {
        // キャリーが発生するケース
        let data = [0xFF, 0xFF, 0x00, 0x01];
        let result = calculate_checksum_rfc1071(&data);

        // 手動計算: 0xFFFF + 0x0001 = 0x10000
        // キャリー処理: (0x10000 & 0xFFFF) + (0x10000 >> 16) = 0x0000 + 0x0001 = 0x0001
        // 1の補数: !0x0001 = 0xFFFE
        let expected = 0xFFFE;
        assert_eq!(result, expected, "RFC 1071 carry test failed");
    }

    #[test]
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
    fn test_checksum_calculation() {
        let src_ip = u32::from_be_bytes([10, 0, 0, 1]);
        let dst_ip = u32::from_be_bytes([10, 0, 0, 2]);

        let mut header = TcpHeader::new(80, 8080, 1000, 2000, tcp_flags::SYN, 1024);
        let tcp_data = b""; // データなし

        let before_checksum = header.checksum;
        println!("Before checksum: 0x{:04X}", before_checksum);

        // チェックサム計算時のデータを記録
        let calc_data = header.prepare_checksum_data(src_ip, dst_ip, tcp_data);
        println!(
            "Calculation data: {:02X?}",
            &calc_data[0..std::cmp::min(calc_data.len(), 32)]
        );

        // 手計算で検証
        let mut manual_sum = 0u32;
        for chunk in calc_data.chunks(2) {
            let word = if chunk.len() == 2 {
                ((chunk[0] as u16) << 8) | (chunk[1] as u16)
            } else {
                (chunk[0] as u16) << 8
            };
            manual_sum += word as u32;
            println!("Adding 0x{:04X}, sum = 0x{:08X}", word, manual_sum);
        }

        // キャリー処理
        while (manual_sum >> 16) != 0 {
            manual_sum = (manual_sum & 0xFFFF) + (manual_sum >> 16);
            println!("Carry processing: sum = 0x{:08X}", manual_sum);
        }

        let manual_checksum = !manual_sum as u16;
        println!("Manual calculated checksum: 0x{:04X}", manual_checksum);

        header.calculate_checksum(src_ip, dst_ip, tcp_data);
        let after_checksum = header.checksum;
        println!("After checksum: 0x{:04X}", after_checksum);

        // チェックサムが計算されて0以外になっているかチェック
        let checksum = header.checksum;
        assert_ne!(checksum, 0);

        // 検証テストのデバッグ
        let verification_result = header.verify_checksum(src_ip, dst_ip, tcp_data);
        println!("Verification result: {}", verification_result);

        // 手動で検証計算を確認
        let all_data = header.prepare_checksum_data(src_ip, dst_ip, tcp_data);
        let manual_result = calculate_checksum_rfc1071(&all_data);
        println!(
            "Manual verification: 0x{:04X} (should be 0xFFFF)",
            manual_result
        );

        // 詳細デバッグ: データ内容を確認
        println!("All data length: {}", all_data.len());
        println!(
            "All data (hex): {:02X?}",
            &all_data[0..std::cmp::min(all_data.len(), 32)]
        );

        // 疑似ヘッダー部分を確認
        let pseudo_header =
            create_pseudo_header(src_ip, dst_ip, (TCP_HEADER_SIZE + tcp_data.len()) as u16);
        println!("Pseudo header: {:02X?}", pseudo_header);

        // TCPヘッダー部分を確認
        let tcp_header_bytes = header.to_bytes();
        println!("TCP header: {:02X?}", tcp_header_bytes);

        // 検証テスト
        assert!(header.verify_checksum(src_ip, dst_ip, tcp_data));
    }

    #[test]
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
        let parsed = TcpHeader::from_bytes(&bytes).expect("Should convert from bytes");

        // 5. チェックサム検証
        assert!(parsed.verify_checksum(src_ip, dst_ip, tcp_data));

        // 6. 値の整合性確認
        let original_source_port = original.source_port;
        let original_destination_port = original.destination_port;
        let parsed_source_port = parsed.source_port;
        let parsed_destination_port = parsed.destination_port;

        assert_eq!(original_source_port, parsed_source_port);
        assert_eq!(original_destination_port, parsed_destination_port);
        assert_eq!(original.get_flags(), parsed.get_flags());
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
