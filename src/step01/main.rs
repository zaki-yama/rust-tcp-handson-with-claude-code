use libc::{AF_INET, IPPROTO_TCP, SOCK_RAW};
use log::{debug, error, info};
use std::net::Ipv4Addr;
use std::os::unix::io::AsRawFd;

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

fn create_raw_socket() -> Result<i32, Box<dyn std::error::Error>> {
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

    fn calculate_checksum(&mut self) {
        // RFC 791のチェックサムアルゴリズム実装
        // 16ビット単位での加算とキャリー処理
        self.checksum = 0;
    }
}
