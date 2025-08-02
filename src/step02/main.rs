use std::net::Ipv4Addr;

const TCP_HEADER_SIZE: usize = 20;

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
/// |  Data |           |U|A|P|R|S|F|                               |
/// | Offset| Reserved  |R|C|S|S|Y|I|            Window             |
/// |       |           |G|K|H|T|N|N|                               |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |           Checksum            |         Urgent Pointer        |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct TcpHeader {
    source_port: u16,
    destination_port: u16,
    sequence_number: u32,
    acknowledgment_number: u32,
    data_offset_and_flags: u16, // Data Offset(4bit) + Reserved(6bit) + Flags(6bit)
    window_size: u16,
    checksum: u16,
    urgent_pointer: u16,
}

/// TCP Flags
#[allow(dead_code)]
mod tcp_flags {
    pub const FIN: u8 = 0x01;
    pub const SYN: u8 = 0x02;
    pub const RST: u8 = 0x04;
    pub const PSH: u8 = 0x08;
    pub const ACK: u8 = 0x10;
    pub const URG: u8 = 0x20;
}

impl TcpHeader {
    /// Create a new TCP header
    ///
    /// # Arguments
    /// * `src_port` - Source port number
    /// * `dst_port` - Destination port number  
    /// * `seq` - Sequence number
    /// * `ack` - Acknowledgment number
    /// * `flags` - TCP flags (6 bits)
    /// * `window` - Window size
    fn new(src_port: u16, dst_port: u16, seq: u32, ack: u32, flags: u8, window: u16) -> Self {
        // TODO: Task B2 - Implement constructor
        // Hints:
        // - Convert port numbers to network byte order with to_be()
        // - Data Offset should be 5 (20 bytes ÷ 4)
        // - Combine data_offset (4bits) + reserved (6bits) + flags (6bits) into 16bit field
        // - Set checksum to 0 initially (will be calculated later)
        todo!("Implement TcpHeader::new()")
    }

    /// Convert TCP header to byte array
    fn to_bytes(&self) -> Vec<u8> {
        // TODO: Task B3 - Implement to_bytes conversion
        // Hints:
        // - Create Vec with capacity 20
        // - Use to_be_bytes() for multi-byte fields
        // - Follow the same pattern as Step1's IpHeader::to_bytes()
        todo!("Implement TcpHeader::to_bytes()")
    }

    /// Parse TCP header from byte array
    fn parse(data: &[u8]) -> Result<Self, &'static str> {
        // TODO: Task C1 - Implement parsing
        // Hints:
        // - Check data length >= 20 bytes
        // - Use u16::from_be_bytes() and u32::from_be_bytes()
        // - Extract bytes in correct order
        todo!("Implement TcpHeader::parse()")
    }

    /// Calculate TCP checksum with pseudo header
    fn calculate_checksum(&mut self, src_ip: u32, dst_ip: u32, tcp_data: &[u8]) {
        // TODO: Task D3 - Implement checksum calculation
        // Hints:
        // - Create pseudo header (12 bytes)
        // - Set checksum field to 0 before calculation
        // - Use RFC 1071 algorithm (same as Step1)
        // - Include pseudo header + TCP header + data
        todo!("Implement checksum calculation")
    }

    /// Verify TCP checksum
    fn verify_checksum(&self, src_ip: u32, dst_ip: u32, tcp_data: &[u8]) -> bool {
        // TODO: Task D4 - Implement checksum verification
        // Hints:
        // - Calculate checksum including existing checksum field
        // - Result should be 0xFFFF if valid
        todo!("Implement checksum verification")
    }

    /// Extract flags from data_offset_and_flags field
    fn get_flags(&self) -> u8 {
        // TODO: Extract lower 6 bits as flags
        (self.data_offset_and_flags & 0x3F) as u8
    }

    /// Extract data offset from data_offset_and_flags field
    fn get_data_offset(&self) -> u8 {
        // TODO: Extract upper 4 bits as data offset
        ((self.data_offset_and_flags >> 12) & 0x0F) as u8
    }
}

/// Create TCP pseudo header for checksum calculation
fn create_pseudo_header(src_ip: u32, dst_ip: u32, tcp_length: u16) -> Vec<u8> {
    // TODO: Task D2 - Implement pseudo header creation
    // Pseudo header format (12 bytes):
    // +--------+--------+--------+--------+
    // |           Source Address          |
    // +--------+--------+--------+--------+
    // |         Destination Address       |
    // +--------+--------+--------+--------+
    // |  zero  |  PTCL  |    TCP Length   |
    // +--------+--------+--------+--------+
    // PTCL = 6 (TCP protocol number)
    todo!("Implement pseudo header creation")
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
    match TcpHeader::parse(&bytes) {
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
