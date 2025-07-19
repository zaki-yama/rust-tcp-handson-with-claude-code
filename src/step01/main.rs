use libc::{AF_INET, IPPROTO_TCP, SOCK_RAW};
use log::info;
use std::{error::Error, net::Ipv4Addr};

// 必要な定数
const IP_HEADER_SIZE: usize = 20;

#[repr(C, packed)]
struct IpHeader {
    // 1バイト目：Version(4bit) + IHL(4bit)
    version_ihl: u8,
    // 2バイト目：Type of Service
    tos: u8,
    // 3-4バイト目：Total Length（ネットワークバイトオーダー）
    length: u16,
    // 5-6バイト目：Identification（パケット識別子）
    id: u16,
    // 7-8バイト目：フラグ(3bit) + フラグメントオフセット(13bit)
    flags_fragment: u16,
    // 9バイト目：Time to Live
    ttl: u8,
    // 10バイト目：上位プロトコル (TCP=6)
    protocol: u8,
    // 11-12バイト目：Header Checksum
    checksum: u16,
    // 13-16バイト目：送信元IPアドレス
    source: u32,
    // 17-20バイト目：宛先IPアドレス
    destination: u32,
}

fn create_raw_socket() -> Result<i32, Box<dyn Error>> {
    // 1. libc::socket(AF_INET, SOCK_RAW, IPPROTO_TCP)でソケット作成
    let socket_fd = unsafe { libc::socket(AF_INET, SOCK_RAW, IPPROTO_TCP) };

    if socket_fd < 0 {
        return Err("Failed to create raw socket".into());
    }
    // 2. IP_HDRINCLオプションを設定してカスタムヘッダーを有効化
    let optval: i32 = 1; // 1 = 有効化
    let result = unsafe {
        libc::setsockopt(
            socket_fd,                                  // ソケットFD
            libc::IPPROTO_IP,                           // IPプロトコルレベル
            libc::IP_HDRINCL,                           // IP_HDRINCLオプション
            &optval as *const _ as *const libc::c_void, // 値へのポインタ
            std::mem::size_of::<i32>() as u32,          // 値のサイズ
        )
    };
    if result < 0 {
        unsafe {
            libc::close(socket_fd);
        }
        return Err("Failed to set IP_HDRINCL".into());
    }

    // 3. エラーハンドリング
    Ok(socket_fd)
}

impl IpHeader {
    fn new(source: Ipv4Addr, dest: Ipv4Addr, data_len: u16) -> Self {
        // 各フィールドを適切な値で初期化
        let total_length = IP_HEADER_SIZE as u16 + data_len;

        Self {
            version_ihl: 0x45,
            tos: 0,
            length: total_length.to_be(),
            id: 0x1234_u16.to_be(),
            flags_fragment: 0x4000_u16.to_be(),
            ttl: 64,
            protocol: 6, // TCP
            checksum: 0, // 後で計算
            source: u32::from(source).to_be(),
            destination: u32::from(dest).to_be(),
        }
    }

    /// RFC 1071のチェックサム計算（共通ロジック）
    fn compute_checksum_sum(&self) -> u32 {
        let mut sum: u32 = 0;

        // IPヘッダーをバイト配列として取得
        let header_bytes = unsafe {
            std::slice::from_raw_parts(
                self as *const _ as *const u8,
                std::mem::size_of::<IpHeader>(),
            )
        };

        // 16bit（2バイト）単位で加算
        for i in (0..IP_HEADER_SIZE).step_by(2) {
            // 2バイトを16bit値として結合（ビッグエンディアン）
            let word = ((header_bytes[i] as u16) << 8) + header_bytes[i + 1] as u16;
            sum += word as u32;
        }

        // キャリーを下位16bitに加算（1の補数演算）
        while (sum >> 16) > 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        sum
    }

    /// RFC 1071のチェックサムアルゴリズム実装
    fn calculate_checksum(&mut self) {
        self.checksum = 0;
        let sum = self.compute_checksum_sum();

        // 全ビット反転（1の補数）
        let checksum = !(sum as u16);

        // ネットワークバイトオーダーで設定
        self.checksum = checksum.to_be();
    }

    /// チェックサム検証
    fn verify_checksum(&self) -> bool {
        let sum = self.compute_checksum_sum();
        // RFC 1071: 検証時は結果が0xFFFFになるべき
        sum as u16 == 0xFFFF
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(20);
        bytes.push(self.version_ihl);
        bytes.push(self.tos);
        bytes.extend_from_slice(&self.length.to_be_bytes());
        bytes.extend_from_slice(&self.id.to_be_bytes());
        bytes.extend_from_slice(&self.flags_fragment.to_be_bytes());
        bytes.push(self.ttl);
        bytes.push(self.protocol);
        bytes.extend_from_slice(&self.checksum.to_be_bytes());
        bytes.extend_from_slice(&self.source.to_be_bytes());
        bytes.extend_from_slice(&self.destination.to_be_bytes());

        return bytes;
    }

    // バイト配列からIpHeaderを作成
    fn from_bytes(data: &[u8]) -> Result<Self, Box<dyn Error>> {
        if data.len() < IP_HEADER_SIZE {
            return Err("Packet too short for IP header".into());
        }

        // バイト配列から各フィールドを抽出
        let version_ihl = data[0];
        let tos = data[1];
        let length = u16::from_be_bytes([data[2], data[3]]);
        let id = u16::from_be_bytes([data[4], data[5]]);
        let flags_fragment = u16::from_be_bytes([data[6], data[7]]);
        let ttl = data[8];
        let protocol = data[9];
        let checksum = u16::from_be_bytes([data[10], data[11]]);
        let source = u32::from_be_bytes([data[12], data[13], data[14], data[15]]);
        let destination = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);

        Ok(IpHeader {
            version_ihl,
            tos,
            length,
            id,
            flags_fragment,
            ttl,
            protocol,
            checksum,
            source,
            destination,
        })
    }

    // フィールドアクセス用メソッド
    fn version(&self) -> u8 {
        (self.version_ihl >> 4) & 0x0F
    }

    fn header_length(&self) -> u8 {
        (self.version_ihl & 0x0F) * 4
    }

    fn source_ip(&self) -> Ipv4Addr {
        Ipv4Addr::from(u32::from_be(self.source))
    }

    fn dest_ip(&self) -> Ipv4Addr {
        Ipv4Addr::from(u32::from_be(self.destination))
    }
}

fn send_packet(
    socket_fd: i32,
    source: Ipv4Addr,
    dest: Ipv4Addr,
    data: &[u8],
) -> Result<(), Box<dyn Error>> {
    // 1. IPヘッダー作成
    let mut ip_header = IpHeader::new(source, dest, data.len() as u16);
    // 2. チェックサム計算
    ip_header.calculate_checksum();

    // 3. データと結合
    let mut packet = ip_header.to_bytes();
    packet.extend_from_slice(data);

    // 4. sendto()で送信
    let dest_sockaddr = libc::sockaddr_in {
        sin_len: std::mem::size_of::<libc::sockaddr_in>() as u8, // macOSで必要
        sin_family: libc::AF_INET as u8,                         // IPv4を指定
        sin_port: 0,                                             // raw socket なのでポート不要
        sin_addr: libc::in_addr {
            s_addr: u32::from(dest).to_be(), // 宛先IPをネットワークバイトオーダーで
        },
        sin_zero: [0; 8], // パディング
    };

    let result = unsafe {
        libc::sendto(
            socket_fd,                                           // ソケットFD
            packet.as_ptr() as *const libc::c_void,              // 送信データ
            packet.len(),                                        // データサイズ
            0,                                                   // フラグ
            &dest_sockaddr as *const _ as *const libc::sockaddr, // 宛先アドレス
            std::mem::size_of::<libc::sockaddr_in>() as u32,     // アドレス構造体サイズ
        )
    };

    if result < 0 {
        return Err("Failed to send packet".into());
    }
    Ok(())
}

fn parse_ip_header(data: &[u8]) -> Result<IpHeader, Box<dyn Error>> {
    let header = IpHeader::from_bytes(data)?;

    // バリデーション
    if header.version() != 4 {
        return Err(format!(
            "Expected IPv4 (version 4), got version {}",
            header.version()
        )
        .into());
    }

    if header.protocol != 6 {
        return Err(format!(
            "Expected TCP (protocol 6), got protocol {}",
            header.protocol
        )
        .into());
    }

    // 各フィールドをログ出力
    info!("=== IP Header Analysis ===");
    info!(
        "Version: {}, Header Length: {} bytes",
        header.version(),
        header.header_length()
    );
    info!("Protocol: {} (TCP)", header.protocol);
    info!("Source: {}", header.source_ip());
    info!("Destination: {}", header.dest_ip());
    info!("Checksum: 0x{:04X}", u16::from_be(header.checksum));

    Ok(header)
}

fn receive_packet(socket_fd: i32) -> Result<Vec<u8>, Box<dyn Error>> {
    // 最大IPパケットサイズ（65535バイト）
    const MAX_PACKET_SIZE: usize = 65535;
    let mut buffer = vec![0u8; MAX_PACKET_SIZE];

    let bytes_received = unsafe {
        libc::recv(
            socket_fd,
            buffer.as_mut_ptr() as *mut libc::c_void,
            buffer.len(),
            0, // ブロッキング受信
        )
    };

    if bytes_received < 0 {
        let errno = unsafe { *libc::__error() };
        return Err(format!("Failed to receive packet: errno {}", errno).into());
    }

    if bytes_received < IP_HEADER_SIZE as isize {
        return Err("Received packet too short for IP header".into());
    }

    info!("Received {} bytes", bytes_received);

    // IPヘッダー部分（最初の20バイト）
    let ip_header_bytes = &buffer[0..IP_HEADER_SIZE];

    // IPヘッダーの解析
    let ip_header = parse_ip_header(ip_header_bytes)?;

    // チェックサム検証
    if !ip_header.verify_checksum() {
        return Err("Invalid IP header checksum".into());
    }

    // 受信したパケット全体を返す
    buffer.truncate(bytes_received as usize);
    Ok(buffer)
}

fn sender_mode() -> Result<(), Box<dyn Error>> {
    info!("=== SENDER MODE ===");

    // raw socket作成
    let socket_fd = create_raw_socket()?;
    info!("Raw socket created successfully (fd: {})", socket_fd);

    // サンプルデータ送信
    let source = Ipv4Addr::new(127, 0, 0, 1);
    let dest = Ipv4Addr::new(127, 0, 0, 1);
    let sample_data = b"Hello, TCP from Step1!";

    info!("Sending packet from {} to {}", source, dest);
    send_packet(socket_fd, source, dest, sample_data)?;
    info!(
        "Sent {} bytes to {}",
        sample_data.len() + IP_HEADER_SIZE,
        dest
    );

    // ソケットクローズ
    unsafe {
        libc::close(socket_fd);
    }

    Ok(())
}

fn receiver_mode() -> Result<(), Box<dyn Error>> {
    info!("=== RECEIVER MODE ===");
    info!("Waiting for incoming packets... (Press Ctrl+C to stop)");

    // raw socket作成
    let socket_fd = create_raw_socket()?;
    info!("Raw socket created successfully (fd: {})", socket_fd);

    // 受信ループ
    loop {
        match receive_packet(socket_fd) {
            Ok(_packet) => {
                info!("Packet processed successfully");
                break; // 1つのパケットを受信したら終了
            }
            Err(e) => {
                // EAGAIN以外のエラーは表示
                let errno = unsafe { *libc::__error() };
                if errno != 35 {
                    // EAGAIN以外
                    info!("Receive error: {}", e);
                }
                // 少し待ってから再試行
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }

    // ソケットクローズ
    unsafe {
        libc::close(socket_fd);
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // ログ初期化
    env_logger::init();

    // コマンドライン引数でモード判定
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "receiver" {
        receiver_mode()
    } else if args.len() > 1 && args[1] == "sender" {
        sender_mode()
    } else {
        // デフォルトは送信モード
        info!("Usage:");
        info!("  Send: sudo cargo run --bin step01 sender");
        info!("  Receive: sudo cargo run --bin step01 receiver");
        info!("");
        info!("Running in sender mode by default...");
        sender_mode()
    }
}
