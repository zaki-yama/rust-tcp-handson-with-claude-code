use std::net::Ipv4Addr;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use log::info;
// Step01とStep02の実装を共通ライブラリから使用
use rust_tcp_handson_with_claude_code::step01::{
    create_raw_socket, get_local_ip, send_packet, IpHeader, IP_HEADER_SIZE, IP_PROTOCOL_TCP,
};
use rust_tcp_handson_with_claude_code::step02::{
    calculate_checksum_rfc1071, tcp_flags, TcpHeader, TCP_HEADER_SIZE,
};

// Raw socketの基本機能（Step1から再利用）
// 実装時にStep1のコードを参考にしてください

#[derive(Debug, Clone, PartialEq)]
pub enum TcpState {
    Closed,
    SynSent,
    Established,
}

#[derive(Debug)]
pub struct TcpConnection {
    socket_fd: i32,
    state: TcpState,
    local_seq: u32,  // 自分のシーケンス番号
    remote_seq: u32, // 相手のシーケンス番号
    local_ip: Ipv4Addr,
    local_port: u16,
    remote_ip: Ipv4Addr,
    remote_port: u16,
}

impl TcpConnection {
    fn new(remote_ip: Ipv4Addr, remote_port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        // - Raw socket作成
        let socket_fd = create_raw_socket()?;

        let local_ip = get_local_ip().unwrap();
        let local_port = Self::choose_local_port();

        Ok(Self {
            socket_fd,
            state: TcpState::Closed,
            local_seq: 0,
            remote_seq: 0,
            local_ip,
            local_port,
            remote_ip,
            remote_port,
        })
    }

    fn connect(&mut self, timeout_secs: u64) -> Result<(), Box<dyn std::error::Error>> {
        // Task F1: 完全な3-way handshakeの実装
        // 1. SYN送信
        // 2. SYN-ACK受信・検証
        // 3. ACK送信
        // 4. 接続完了
        todo!("Task F1: 3-way handshake全体の流れを実装してください")
    }

    fn send_tcp_packet(
        &self,
        tcp_header_bytes: &[u8],
        data: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source = self.local_ip;
        let dest = Ipv4Addr::from(self.remote_ip);
        let data_len = tcp_header_bytes.len() + data.len();

        let ip_header = IpHeader::new(source, dest, data_len as u16);

        let mut packet = Vec::new();
        packet.extend_from_slice(&ip_header.to_bytes());
        packet.extend_from_slice(tcp_header_bytes);
        packet.extend_from_slice(data);

        let dest_sockaddr = libc::sockaddr_in {
            sin_len: std::mem::size_of::<libc::sockaddr_in>() as u8,
            sin_family: libc::AF_INET as u8,
            sin_port: 0,
            sin_addr: libc::in_addr {
                s_addr: u32::from(self.remote_ip), // Ipv4Addr -> u32（ネットワークバイトオーダー）
            },
            sin_zero: [0; 8],
        };

        let result = unsafe {
            libc::sendto(
                self.socket_fd,                                      // ソケットFD
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


    /// Task C2: SYN送信機能
    fn send_syn(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.local_seq = generate_isn();
        let syn_packet = self.create_syn_packet()?;
        self.send_tcp_packet(&syn_packet, &[])?;
        self.state = TcpState::SynSent;
        println!("SYN sent: seq={}", self.local_seq);
        Ok(())
    }

    fn create_syn_packet(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut header = TcpHeader::new(
            self.local_port,
            self.remote_port,
            self.local_seq, // ISN
            0,              // ACK番号は0
            tcp_flags::SYN, // SYNフラグ（bit 1）
            8192,           // ウィンドウサイズ
        );

        // チェックサム計算
        header.calculate_checksum(
            u32::from(self.local_ip),
            u32::from(self.remote_ip),
            &[], // データなし
        );

        Ok(header.to_bytes())
    }

    fn receive_packet_timeout(
        &self,
        timeout_secs: u64,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let start = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            // ノンブロッキング受信を試行
            match self.try_receive_packet() {
                Ok(data) => return Ok(data),
                Err(_) => {
                    if start.elapsed() > timeout {
                        return Err("Timeout".into());
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }
    }

    fn parse_received_packet(&self, data: &[u8]) -> Result<TcpHeader, Box<dyn std::error::Error>> {
        // Task D2: 受信パケット解析
        // - IPヘッダー長計算
        if data.len() < IP_HEADER_SIZE {
            return Err("Packet too short".into());
        }

        // IPプロトコルチェック
        let protocol = data[9];
        if protocol != IP_PROTOCOL_TCP {
            return Err("Not a TCP packet".into());
        }

        // IP ヘッダーの 1 バイト目は version (4bit) + IHL: Internet Header Length (4bit)
        // IHL の情報から IP ヘッダーのバイト数を算出するため下位 4bit の IHL だけを抽出
        let ip_header_len = ((data[0] & 0x0F) * 4) as usize;
        if data.len() < ip_header_len + 20 {
            return Err("TCP header incomplete".into());
        }

        // TCPヘッダー部分を抽出
        let tcp_data = &data[ip_header_len..];

        let tcp_header = TcpHeader::from_bytes(tcp_data)?;
        // ポート番号チェック
        if tcp_header.get_destination_port() != self.local_port {
            return Err("Packet not for this connection".into());
        }
        Ok(tcp_header)
    }

    fn is_correct_syn_ack(&self, tcp_header: &TcpHeader) -> bool {
        // Task D3: SYN-ACK検証
        // - SYN + ACKフラグチェック
        let flags = tcp_header.get_flags();
        let has_syn_ack = (flags & tcp_flags::SYN) != 0 && (flags & tcp_flags::ACK) != 0;

        // - ACK番号の正確性チェック (local_seq + 1)
        let has_correct_ack = tcp_header.get_ack_number() == self.local_seq + 1;

        // - ポート番号チェック
        let has_correct_ports = tcp_header.get_source_port() == self.remote_port
            && tcp_header.get_destination_port() == self.local_port;

        has_syn_ack && has_correct_ack && has_correct_ports
    }

    fn send_ack(&mut self, ack_number: u32) -> Result<(), Box<dyn std::error::Error>> {
        // Task E2: ACK送信
        // - ACKパケット作成
        // - 送信
        // - 状態管理
        todo!("Task E2: ACK送信機能を実装してください")
    }

    fn create_ack_packet(&self, ack_number: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Task E1: ACKパケット構築
        // - ACKフラグ付きTCPヘッダー作成
        // - 正しいseq/ack番号設定

        let mut header = TcpHeader::new(
            self.local_port,
            self.remote_port,
            self.local_seq + 1, // SYN 送信後なので+1
            ack_number,
            tcp_flags::ACK, // ACKフラグ
            8192,           // ウィンドウサイズ
        );

        // チェックサム計算
        header.calculate_checksum(
            u32::from(self.local_ip),
            u32::from(self.remote_ip),
            &[], // データなし
        );

        Ok(header.to_bytes())
    }

    fn complete_handshake(&mut self) {
        // Task E3: 接続確立完了
        // - 状態をESTABLISHEDに変更
        // - シーケンス番号更新
        todo!("Task E3: 接続確立完了処理を実装してください")
    }

    fn is_connected(&self) -> bool {
        // Task E4: 接続確認
        self.state == TcpState::Established
    }

    /// 動的にローカルポートを選択
    fn choose_local_port() -> u16 {
        use std::net::TcpListener;

        // ポート0を指定することでOSが自動的に利用可能なポートを選択
        if let Ok(listener) = TcpListener::bind("127.0.0.1:0") {
            if let Ok(addr) = listener.local_addr() {
                return addr.port();
            }
        }

        // フォールバック: 49152-65535の範囲からランダム選択
        49152
            + (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u16
                % (65535 - 49152))
    }

    fn try_receive_packet(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // 最大IPパケットサイズ（65535バイト）
        const MAX_PACKET_SIZE: usize = 65535;
        let mut buffer = vec![0u8; MAX_PACKET_SIZE];

        let bytes_received = unsafe {
            libc::recv(
                self.socket_fd,
                buffer.as_mut_ptr() as *mut libc::c_void,
                buffer.len(),
                libc::MSG_DONTWAIT, // ノンブロッキングフラグ
            )
        };

        if bytes_received < 0 {
            let errno = unsafe { *libc::__error() };

            // ノンブロッキングで利用可能なデータがない場合
            if errno == libc::EAGAIN || errno == libc::EWOULDBLOCK {
                info!("Received {} bytes", bytes_received);
                return Err("No data available".into());
            }
            return Err(format!("Failed to receive packet: errno {}", errno).into());
        }

        if bytes_received < IP_HEADER_SIZE as isize {
            return Err("Received packet too short for IP header".into());
        }

        info!("Received {} bytes", bytes_received);

        // 受信したパケット全体を返す
        buffer.truncate(bytes_received as usize);
        Ok(buffer)
    }
}

/// ISN: The Initial Sequence Number
fn generate_isn() -> u32 {
    // RFC 6528推奨の安全な初期シーケンス番号生成
    // 簡易実装: 現在時刻 + ランダム値
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    // 注意: 本来はrand crateを使用しますが、簡易実装として現在時刻を使用
    // 実際の実装では crypto-secure randomを使用してください
    now.wrapping_add(12345) // 簡易的なランダム要素
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Step 3: 3-way Handshake Implementation");
    println!("========================================");

    // ログ初期化（必要に応じて）
    // env_logger::init();

    // デモ実行例
    println!("Demo: Attempting 3-way handshake with localhost:80");

    let remote_ip = Ipv4Addr::new(127, 0, 0, 1); // localhost

    // 注意: 実際にはlocalhostの80番ポートにHTTPサーバーが動いている必要があります
    // テスト用にnetcatを使用: nc -l 80 （別ターミナルで実行）

    let mut conn = TcpConnection::new(remote_ip, 80)?;

    println!("Initial state: {:?}", conn.state);

    match conn.connect(5) {
        Ok(_) => {
            println!("✅ Successfully established TCP connection!");
            println!("Final state: {:?}", conn.state);
            println!("Connection details:");
            println!("  Local seq: {}", conn.local_seq);
            println!("  Remote seq: {}", conn.remote_seq);
        }
        Err(e) => {
            println!("❌ Connection failed: {}", e);
            println!("Current state: {:?}", conn.state);
        }
    }

    println!("\n📝 Next steps:");
    println!("1. 各Task（A1-F4）を順番に実装してください");
    println!("2. テストを実行して動作確認してください: cargo test");
    println!("3. Wiresharkでパケットキャプチャを確認してください");
    println!("4. 学習した内容をLEARNING_LOG.mdに記録してください");

    Ok(())
}

#[cfg(test)]
mod tests;
