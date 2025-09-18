pub const TCP_HEADER_SIZE: usize = 20;

/// TCP Header Structure (RFC 9293 Section 3.1 - 2022 updated standard)
///
/// 0                   1                   2                   3
/// 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |          Source Port          |       Destination Port        |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                        Sequence Number                        |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                    Acknowledgment Number                      |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |  Data |       |C|E|U|A|P|R|S|F|                               |
/// | Offset| Rsrvd |W|C|R|C|S|S|Y|I|            Window             |
/// |       |       |R|E|G|K|H|T|N|N|                               |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |           Checksum            |         Urgent Pointer        |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                           [Options]                           |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                                                               :
/// :                             Data                              :
/// :                                                               |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct TcpHeader {
    source_port: u16,
    destination_port: u16,
    // シーケンス番号。初期値はコネクション確立時に乱数で決定される
    sequence_number: u32,
    // 確認応答番号。次に受信すべきデータのシーケンス番号
    acknowledgment_number: u32,
    // Data Offset(4bit) + Reserved(4bit) + Flags(8bit)
    data_offset_and_flags: u16,
    window_size: u16,
    checksum: u16,
    urgent_pointer: u16,
}

/// TCP Flags (RFC 9293 Section 3.1)
pub mod tcp_flags {
    pub const FIN: u8 = 0x01; // No more data from sender
    pub const SYN: u8 = 0x02; // Synchronize sequence numbers
    pub const RST: u8 = 0x04; // Reset the connection
    pub const PSH: u8 = 0x08; // Push function
    pub const ACK: u8 = 0x10; // Acknowledgment field is significant
    pub const URG: u8 = 0x20; // Urgent pointer field is significant
    pub const ECE: u8 = 0x40; // ECN-Echo
    pub const CWR: u8 = 0x80; // Congestion Window Reduced
}

impl TcpHeader {
    /// Create a new TCP header
    ///
    /// # Arguments
    /// * `src_port` - Source port number
    /// * `dst_port` - Destination port number
    /// * `seq` - Sequence number
    /// * `ack` - Acknowledgment number
    /// * `flags` - TCP flags (8 bits)
    /// * `window` - Window size
    pub fn new(src_port: u16, dst_port: u16, seq: u32, ack: u32, flags: u8, window: u16) -> Self {
        Self {
            source_port: src_port,
            destination_port: dst_port,
            sequence_number: seq,
            acknowledgment_number: ack,
            // data offset は 5
            data_offset_and_flags: (5u16 << 12) | (flags as u16),
            window_size: window,
            urgent_pointer: 0,
            checksum: 0,
        }
    }

    /// Convert TCP header to byte array
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(20);
        bytes.extend_from_slice(&self.source_port.to_be_bytes());
        bytes.extend_from_slice(&self.destination_port.to_be_bytes());
        bytes.extend_from_slice(&self.sequence_number.to_be_bytes());
        bytes.extend_from_slice(&self.acknowledgment_number.to_be_bytes());
        bytes.extend_from_slice(&self.data_offset_and_flags.to_be_bytes());
        bytes.extend_from_slice(&self.window_size.to_be_bytes());
        bytes.extend_from_slice(&self.checksum.to_be_bytes());
        bytes.extend_from_slice(&self.urgent_pointer.to_be_bytes());
        bytes
    }

    /// Parse TCP header from byte array
    pub fn from_bytes(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < TCP_HEADER_SIZE {
            return Err("Data too short for TCP header");
        }

        Ok(Self {
            source_port: u16::from_be_bytes([data[0], data[1]]),
            destination_port: u16::from_be_bytes([data[2], data[3]]),
            sequence_number: u32::from_be_bytes([data[4], data[5], data[6], data[7]]),
            acknowledgment_number: u32::from_be_bytes([data[8], data[9], data[10], data[11]]),
            data_offset_and_flags: u16::from_be_bytes([data[12], data[13]]),
            window_size: u16::from_be_bytes([data[14], data[15]]),
            checksum: u16::from_be_bytes([data[16], data[17]]),
            urgent_pointer: u16::from_be_bytes([data[18], data[19]]),
        })
    }

    /// 内部ヘルパー: チェックサム計算用の全データを準備
    fn prepare_checksum_data(&self, src_ip: u32, dst_ip: u32, tcp_data: &[u8]) -> Vec<u8> {
        let pseudo_header =
            create_pseudo_header(src_ip, dst_ip, (TCP_HEADER_SIZE + tcp_data.len()) as u16);
        let tcp_header_bytes = self.to_bytes();

        let mut all_data = Vec::new();
        all_data.extend_from_slice(&pseudo_header);
        all_data.extend_from_slice(&tcp_header_bytes);
        all_data.extend_from_slice(tcp_data);
        all_data
    }

    /// Calculate TCP checksum with pseudo header
    pub fn calculate_checksum(&mut self, src_ip: u32, dst_ip: u32, tcp_data: &[u8]) {
        self.checksum = 0; // 先にクリア
        let all_data = self.prepare_checksum_data(src_ip, dst_ip, tcp_data);
        self.checksum = calculate_checksum_rfc1071(&all_data);
    }

    /// Verify TCP checksum
    fn verify_checksum(&self, src_ip: u32, dst_ip: u32, tcp_data: &[u8]) -> bool {
        // checksumはそのまま（クリアしない）
        let all_data = self.prepare_checksum_data(src_ip, dst_ip, tcp_data);
        let result = calculate_1s_complement_sum(&all_data);
        result == 0xFFFF // 正しければ0xFFFF
    }

    /// Extract flags from data_offset_and_flags field
    pub fn get_flags(&self) -> u8 {
        // Extract lower 8 bits as flags
        (self.data_offset_and_flags & 0xFF) as u8
    }

    /// Extract data offset from data_offset_and_flags field
    fn get_data_offset(&self) -> u8 {
        ((self.data_offset_and_flags >> 12) & 0x0F) as u8
    }

    pub fn get_source_port(&self) -> u16 {
        self.source_port
    }

    pub fn get_destination_port(&self) -> u16 {
        self.destination_port
    }

    pub fn get_ack_number(&self) -> u32 {
        self.acknowledgment_number
    }
}

/// 1の補数和を計算（キャリー処理まで、補数演算なし）
fn calculate_1s_complement_sum(data: &[u8]) -> u16 {
    let mut sum = 0u32; // オーバーフローを避けるため32ビット

    // 16ビットずつ処理
    for chunk in data.chunks(2) {
        let word = if chunk.len() == 2 {
            // 2バイトある場合：ビッグエンディアンで16ビット値を作成
            ((chunk[0] as u16) << 8) | (chunk[1] as u16)
        } else {
            // 1バイトしかない場合（奇数長）：上位バイトに配置
            (chunk[0] as u16) << 8
        };
        sum += word as u32;
    }

    // キャリーを処理
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    sum as u16
}

/// RFC 1071チェックサム計算（1の補数演算を含む）
pub fn calculate_checksum_rfc1071(data: &[u8]) -> u16 {
    let sum = calculate_1s_complement_sum(data);
    !sum // 1の補数を取る
}

/// Create TCP pseudo header for checksum calculation
/// +--------+--------+--------+--------+
/// |           Source Address          |
/// +--------+--------+--------+--------+
/// |         Destination Address       |
/// +--------+--------+--------+--------+
/// |  zero  |  PTCL  |    TCP Length   |
/// +--------+--------+--------+--------+
/// PTCL = 6 (TCP protocol number)
fn create_pseudo_header(src_ip: u32, dst_ip: u32, tcp_length: u16) -> Vec<u8> {
    // Pseudo header format (12 bytes):
    let mut bytes = Vec::with_capacity(12);
    bytes.extend_from_slice(&src_ip.to_be_bytes());
    bytes.extend_from_slice(&dst_ip.to_be_bytes());
    bytes.push(0);
    bytes.push(6); // PTCL
    bytes.extend_from_slice(&tcp_length.to_be_bytes());
    bytes
}

fn main() {
    println!("Step 2: TCP Header Analysis");
    println!("==========================");

    // Task B4: Basic functionality test
    println!("\n1. Creating TCP header...");

    // Example: HTTP request (port 80 -> 12345, SYN flag)
    let header = TcpHeader::new(80, 12345, 1000, 0, tcp_flags::SYN, 8192);
    println!("TCP Header created: {:?}", header);

    println!("\n2. Converting to bytes...");
    let bytes = header.to_bytes();
    println!("TCP Header bytes ({} bytes): {:02X?}", bytes.len(), bytes);

    println!("\n3. Parsing from bytes...");
    match TcpHeader::from_bytes(&bytes) {
        Ok(parsed) => {
            println!("Parse successful: {:?}", parsed);
            println!("Source port: {}", u16::from_be(parsed.source_port));
            println!(
                "Destination port: {}",
                u16::from_be(parsed.destination_port)
            );
            println!("Sequence number: {}", u32::from_be(parsed.sequence_number));
            println!("Flags: 0b{:06b}", parsed.get_flags());
        }
        Err(e) => println!("Parse failed: {}", e),
    }

    println!("\n4. Checksum calculation (TODO)...");
    // This will be implemented in Phase D

    println!("\nStep 2 implementation in progress...");
}

#[cfg(test)]
mod tests; // Load tests from tests.rs file
