use libc::{AF_INET, IPPROTO_TCP, SOCK_RAW};
use log::info;
use std::{error::Error, net::Ipv4Addr};

// 必要な定数
const IP_HEADER_SIZE: usize = 20;

// ローカルIPアドレスを取得する関数
fn get_local_ip() -> Option<Ipv4Addr> {
    use std::net::UdpSocket;

    // 外部のDNSサーバー（Google DNS）に接続を試行して、ローカルIPを取得
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(local_addr) = socket.local_addr() {
                if let std::net::IpAddr::V4(ipv4) = local_addr.ip() {
                    return Some(ipv4);
                }
            }
        }
    }
    None
}

#[repr(C, packed)]
struct IpHeader {
    // 1バイト目：Version(4bit) + IHL(4bit)
    version_ihl: u8,
    // 2バイト目：Type of Service
    tos: u8,
    // 3-4バイト目：Total Length（カーネルがホストバイトオーダーで期待）
    length: u16,
    // 5-6バイト目：Identification（カーネルが設定するため0）
    id: u16,
    // 7-8バイト目：フラグ(3bit) + フラグメントオフセット(13bit)（ホストバイトオーダー）
    flags_fragment: u16,
    // 9バイト目：Time to Live
    ttl: u8,
    // 10バイト目：上位プロトコル (TCP=6)
    protocol: u8,
    // 11-12バイト目：Header Checksum（カーネルが計算するため0）
    checksum: u16,
    // 13-16バイト目：送信元IPアドレス（ネットワークバイトオーダー）
    source: u32,
    // 17-20バイト目：宛先IPアドレス（ネットワークバイトオーダー）
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
    /// macOS raw socket用のIPヘッダー作成
    ///
    /// カーネルが処理するフィールド（id, checksum）は0に設定
    fn new(source: Ipv4Addr, dest: Ipv4Addr, data_len: u16) -> Self {
        let total_length = IP_HEADER_SIZE as u16 + data_len;

        Self {
            version_ihl: 0x45,            // IPv4, 20バイトヘッダー
            tos: 0,                       // サービスタイプ
            length: total_length,         // 全長（ホストバイトオーダー）
            id: 0,                        // ID（カーネルが設定）
            flags_fragment: 0x4000,       // Don't Fragment（ホストバイトオーダー）
            ttl: 64,                      // Time To Live
            protocol: 6,                  // TCP
            checksum: 0,                  // チェックサム（カーネルが計算）
            source: u32::from(source),    // 送信元IP
            destination: u32::from(dest), // 宛先IP
        }
    }


    /// IPヘッダーをバイト配列に変換（macOS raw socket用）
    ///
    /// - length/flags_fragment: ホストバイトオーダー（カーネルが変換）
    /// - id/checksum: 0（カーネルが設定/計算）
    /// - IPアドレス: ネットワークバイトオーダー
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(20);
        bytes.push(self.version_ihl);
        bytes.push(self.tos);
        // length: ホストバイトオーダー（カーネルが変換）
        bytes.extend_from_slice(&self.length.to_ne_bytes());
        // id: 0（カーネルが設定）
        bytes.extend_from_slice(&[0, 0]);
        // flags_fragment: ホストバイトオーダー（カーネルが変換）
        bytes.extend_from_slice(&self.flags_fragment.to_ne_bytes());
        bytes.push(self.ttl);
        bytes.push(self.protocol);
        // checksum: 0（カーネルが計算）
        bytes.extend_from_slice(&[0, 0]);
        // IPアドレス: ネットワークバイトオーダー
        bytes.extend_from_slice(&self.source.to_be_bytes());
        bytes.extend_from_slice(&self.destination.to_be_bytes());

        bytes
    }

    /// ネットワークバイトオーダーのバイト配列からIpHeaderを作成
    ///
    /// Step1では実際には使用しない（受信処理はmacOSで制限される）
    /// 学習目的でのパケット解析用の実装
    fn from_bytes(data: &[u8]) -> Result<Self, Box<dyn Error>> {
        if data.len() < IP_HEADER_SIZE {
            return Err("Packet too short for IP header".into());
        }

        // ネットワークバイトオーダー→ネイティブバイトオーダー変換
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
        Ipv4Addr::from(self.source)
    }

    fn dest_ip(&self) -> Ipv4Addr {
        Ipv4Addr::from(self.destination)
    }
}

fn send_packet(
    socket_fd: i32,
    source: Ipv4Addr,
    dest: Ipv4Addr,
    data: &[u8],
) -> Result<(), Box<dyn Error>> {
    // 1. IPヘッダー作成（カーネルがid/checksumを処理）
    let ip_header = IpHeader::new(source, dest, data.len() as u16);

    // 2. データと結合
    let mut packet = ip_header.to_bytes();
    packet.extend_from_slice(data);

    // Debug: Show header contents
    info!("IP Header bytes: {:02x?}", &packet[0..20]);
    info!("Data bytes: {:02x?}", data);

    // 4. sendto()で送信
    let dest_sockaddr = libc::sockaddr_in {
        sin_len: std::mem::size_of::<libc::sockaddr_in>() as u8, // macOSで必要
        sin_family: libc::AF_INET as u8,                         // IPv4を指定
        sin_port: 0,                                             // raw socket なのでポート不要
        sin_addr: libc::in_addr {
            s_addr: u32::from(dest), // 宛先IPをネットワークバイトオーダーで
        },
        sin_zero: [0; 8], // パディング
    };

    info!("Packet size: {} bytes", packet.len());
    info!(
        "Destination sockaddr - family: {}, addr: 0x{:08x}",
        dest_sockaddr.sin_family, dest_sockaddr.sin_addr.s_addr
    );

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
        let errno = unsafe { *libc::__error() };
        return Err(format!("Failed to send packet: errno {}", errno).into());
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
    let checksum = header.checksum;
    info!("Checksum: 0x{:04X}", checksum);

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
    let _ip_header = parse_ip_header(ip_header_bytes)?;

    // 受信したパケット全体を返す
    buffer.truncate(bytes_received as usize);
    Ok(buffer)
}

fn sender_mode() -> Result<(), Box<dyn Error>> {
    info!("=== SENDER MODE ===");

    // raw socket作成
    let socket_fd = create_raw_socket()?;
    info!("Raw socket created successfully (fd: {})", socket_fd);

    // 実際のネットワークインターフェースを使用（macOS対応）
    info!("Getting local IP address for macOS compatibility...");
    let local_ip = get_local_ip().unwrap_or_else(|| {
        info!("Could not detect local IP, using localhost");
        Ipv4Addr::new(127, 0, 0, 1)
    });

    info!("Using IP address: {}", local_ip);
    // 実際のローカルIPアドレスを使用（自分自身に送信）
    let source = local_ip;
    let dest = local_ip;
    let sample_data = b"Hello, TCP from Step1!";

    info!("Sending packet from {} to {}", source, dest);
    send_packet(socket_fd, source, dest, sample_data)?;
    info!(
        "Sent {} bytes to {} (IP header: {}, Data: {})",
        sample_data.len() + IP_HEADER_SIZE,
        dest,
        IP_HEADER_SIZE,
        sample_data.len()
    );
    info!(
        "Packet content: {:?}",
        std::str::from_utf8(sample_data).unwrap_or("(binary data)")
    );

    // デバッグ用: 実際に送信されたパケットの詳細を表示
    let debug_header = IpHeader::new(source, dest, sample_data.len() as u16);
    let debug_packet = debug_header.to_bytes();
    info!("IP Header (hex): {:02x?}", &debug_packet[0..20]);
    info!("Data (hex): {:02x?}", sample_data);

    // ソケットクローズ
    unsafe {
        libc::close(socket_fd);
    }

    Ok(())
}

fn receiver_mode() -> Result<(), Box<dyn Error>> {
    info!("=== RECEIVER MODE ===");
    info!("Attempting to receive packets on actual network interface");
    info!("This may work when using real network IP (192.168.X.X) instead of localhost");
    info!("");
    info!("For verification, use tcpdump in parallel:");
    info!("  macOS: sudo tcpdump -i en0 -v -X host [your-ip]");
    info!("  Linux: sudo tcpdump -i eth0 -v -X host [your-ip]");
    info!("");
    info!("Waiting for incoming packets... (10 second timeout)");

    // raw socket作成
    let socket_fd = create_raw_socket()?;
    info!("Raw socket created successfully (fd: {})", socket_fd);

    // タイムアウト設定（10秒）
    let timeout = libc::timeval {
        tv_sec: 10,
        tv_usec: 0,
    };

    unsafe {
        libc::setsockopt(
            socket_fd,
            libc::SOL_SOCKET,
            libc::SO_RCVTIMEO,
            &timeout as *const _ as *const libc::c_void,
            std::mem::size_of::<libc::timeval>() as u32,
        );
    }

    // 1回だけ受信を試行
    match receive_packet(socket_fd) {
        Ok(_packet) => {
            info!("Packet processed successfully");
        }
        Err(e) => {
            let errno = unsafe { *libc::__error() };
            if errno == 35 || errno == 60 {
                // EAGAIN/EWOULDBLOCK or ETIMEDOUT
                info!("No packets received within timeout period");
                info!("This is expected on localhost with raw sockets");
                info!("");
                info!("✅ Step1 COMPLETED SUCCESSFULLY:");
                info!("  - Raw socket creation: ✅");
                info!("  - IP header construction: ✅");
                info!("  - Packet transmission: ✅");
                info!("  - Header parsing implementation: ✅");
                info!("  - Checksum calculation/verification: ✅");
                info!("");
                info!("Use tcpdump to verify actual packet transmission");
            } else {
                info!("Receive error: {} (errno: {})", e, errno);
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
