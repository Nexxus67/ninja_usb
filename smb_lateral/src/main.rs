use smbclient::SmbClient;
use std::env;
use std::net::TcpStream;
use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <target_ip> <username> <ntlm_hash>", args[0]);
        return;
    }

    let ip = &args[1];
    let user = &args[2];
    let hash = &args[3];

    if TcpStream::connect_timeout(&format!("{}:445", ip).parse().unwrap(), Duration::from_secs(2)).is_err() {
        return; // host unreachable or port closed
    }

    if let Ok(mut client) = SmbClient::new(ip, user, hash) {
        if client.connect().is_ok() {
            if let Ok(shares) = client.list_shares() {
                for s in shares {
                    println!("{}", s); // 
                }
            }
        }
    }
}


// cargo run --release -- 192.168.0.10 administrator 8846f7eaee8fb117ad06bdd830b7586c