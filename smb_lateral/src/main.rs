use clap::Parser;
use std::net::{SocketAddr, ToSocketAddrs, TcpStream};
use std::time::{Duration, Instant};
use std::{thread, process};
use anyhow::{Context, Result};
use zeroize::Zeroizing;
use log::{info, warn, error};

use smbclient::SmbClient;

#[derive(Parser)]
struct Args {
    target_ip: String,
    username: String,
    ntlm_hash: String,
    #[clap(long, default_value_t = 445)]
    port: u16,
    #[clap(long, default_value_t = 2)]
    connect_timeout_sec: u64,
    #[clap(long, default_value_t = 3)]
    retries: u8,
    #[clap(long, default_value_t = 500)]
    retry_backoff_ms: u64,
}

fn resolve_sockaddr(target: &str, port: u16) -> Result<SocketAddr> {
    let addr_iter = (target, port).to_socket_addrs()
        .with_context(|| format!("resolving {}:{}", target, port))?;
    addr_iter
        .into_iter()
        .next()
        .context("no socket address found")
}

fn try_connect(addr: &SocketAddr, timeout: Duration) -> bool {
    TcpStream::connect_timeout(addr, timeout).is_ok()
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    let mut ntlm_hash = Zeroizing::new(args.ntlm_hash);
    if ntlm_hash.len() < 32 {
        error!("ntlm_hash looks too short");
        process::exit(2);
    }

    let addr = match resolve_sockaddr(&args.target_ip, args.port) {
        Ok(a) => a,
        Err(e) => {
            error!("address resolution failed: {}", e);
            process::exit(3);
        }
    };

    let timeout = Duration::from_secs(args.connect_timeout_sec);
    let mut attempt: u8 = 0;
    let start = Instant::now();
    let mut last_err = None;

    while attempt <= args.retries {
        attempt += 1;
        info!("attempt {}/{} connecting to {} (timeout {:?})", attempt, args.retries, addr, timeout);
        if try_connect(&addr, timeout) {
            info!("tcp connect ok");
            match SmbClient::new(&args.target_ip, &args.username, &ntlm_hash) {
                Ok(mut client) => {
                    if let Err(e) = client.connect() {
                        warn!("smb connect failed: {}", e);
                        last_err = Some(e.into());
                    } else {
                        match client.list_shares() {
                            Ok(shares) => {
                                for s in shares.into_iter().take(500) {
                                    println!("{}", s);
                                }
                                info!("shares listed (duration {:.2?})", start.elapsed());
                                process::exit(0);
                            }
                            Err(e) => {
                                warn!("list_shares failed: {}", e);
                                last_err = Some(e.into());
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("SmbClient::new failed: {}", e);
                    last_err = Some(e.into());
                }
            }
        } else {
            warn!("tcp connect timeout/reject on attempt {}", attempt);
        }

        if attempt > args.retries {
            break;
        }
        thread::sleep(Duration::from_millis(args.retry_backoff_ms * attempt as u64));
    }

    if let Some(e) = last_err {
        error!("final error: {:?}", e);
    } else {
        error!("unable to reach host or no shares found");
    }
    process::exit(1);
}


// cargo run --release -- 192.168.0.10 administrator 8846f7eaee8fb117ad06bdd830b7586c