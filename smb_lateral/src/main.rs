use clap::Parser;
use std::net::{SocketAddr, ToSocketAddrs, TcpStream};
use std::time::{Duration, Instant};
use std::{thread, process};
use anyhow::{Context, Result};
use zeroize::Zeroizing;
use log::{info, warn, error, LevelFilter};
use env_logger::Builder;
use serde_json::json;

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
    #[clap(long)]
    json: bool,
    #[clap(long)]
    quiet: bool,
}

fn resolve_sockaddrs(target: &str, port: u16) -> Result<Vec<SocketAddr>> {
    let addr_iter = (target, port).to_socket_addrs()
        .with_context(|| format!("resolving {}:{}", target, port))?;
    let addrs: Vec<_> = addr_iter.into_iter().collect();
    if addrs.is_empty() {
        anyhow::bail!("no socket addresses found");
    }
    Ok(addrs)
}

fn try_connect(addr: &SocketAddr, timeout: Duration) -> bool {
    TcpStream::connect_timeout(addr, timeout).is_ok()
}

fn main() {
    let args = Args::parse();

    let mut builder = Builder::new();
    if args.quiet {
        builder.filter_level(LevelFilter::Error);
    } else {
        builder.parse_default_env();
    }
    builder.init();

    let mut ntlm_hash = Zeroizing::new(args.ntlm_hash);
    if ntlm_hash.len() < 32 {
        error!("ntlm_hash looks too short");
        process::exit(2);
    }

    let decoded = match hex::decode(&*ntlm_hash) {
        Ok(v) => Zeroizing::new(v),
        Err(_) => {
            error!("ntlm_hash is not valid hex");
            process::exit(2);
        }
    };

    let addrs = match resolve_sockaddrs(&args.target_ip, args.port) {
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
        info!("attempt {}/{} connecting to {} (timeout {:?})", attempt, args.retries, args.port, timeout);

        let mut connected = false;
        let mut connect_addr = None;
        for addr in &addrs {
            info!("probing {}", addr);
            if try_connect(addr, timeout) {
                connected = true;
                connect_addr = Some(*addr);
                break;
            } else {
                warn!("tcp connect timeout/reject on {}", addr);
            }
        }

        if connected {
            let target_host = &args.target_ip;
            match SmbClient::new(target_host, &args.username, &*ntlm_hash) {
                Ok(mut client) => {
                    if let Err(e) = client.connect() {
                        warn!("smb connect failed: {}", e);
                        last_err = Some(e.into());
                    } else {
                        match client.list_shares() {
                            Ok(shares) => {
                                let shares_limited: Vec<String> = shares.into_iter().take(500).collect();
                                if args.json {
                                    let out = json!({
                                        "target": target_host,
                                        "addr": connect_addr.map(|a| a.to_string()),
                                        "shares": shares_limited,
                                        "duration_ms": start.elapsed().as_millis()
                                    });
                                    println!("{}", out.to_string());
                                } else {
                                    for s in shares_limited.iter() {
                                        println!("{}", s);
                                    }
                                    info!("shares listed (duration {:.2?})", start.elapsed());
                                }
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
        }

        if attempt > args.retries {
            break;
        }
        thread::sleep(Duration::from_millis(args.retry_backoff_ms * attempt as u64));
    }

    if let Some(e) = last_err {
        if args.json {
            let out = json!({
                "error": format!("{:?}", e)
            });
            println!("{}", out.to_string());
        } else {
            error!("final error: {:?}", e);
        }
    } else {
        if args.json {
            let out = json!({
                "error": "unable to reach host or no shares found"
            });
            println!("{}", out.to_string());
        } else {
            error!("unable to reach host or no shares found");
        }
    }
    process::exit(1);
}



// cargo run --release -- 192.168.0.10 administrator 8846f7eaee8fb117ad06bdd830b7586c