use clap::Parser;
use std::net::{SocketAddr, ToSocketAddrs, TcpStream};
use std::time::{Duration, Instant};
use std::{thread, process, io::Write};
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
    #[clap(long, default_value_t = 0)]
    rate_limit_ms: u64,
    #[clap(long)]
    json: bool,
    #[clap(long)]
    quiet: bool,
    #[clap(long)]
    audit_log: Option<String>,
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

fn audit_append(path: &Option<String>, entry: &str) {
    if let Some(p) = path {
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(p) {
            let _ = writeln!(f, "{}", entry);
        }
    }
}

fn is_auth_error(e: &anyhow::Error) -> bool {
    let s = e.to_string().to_lowercase();
    s.contains("auth") || s.contains("logon") || s.contains("access denied") || s.contains("status_logon")
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

    // Validate hex format and keep decoded in Zeroizing (for hygiene), though we send hex string to crate.
    let _decoded = match hex::decode(&*ntlm_hash) {
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
    let start = Instant::now();
    let mut last_err: Option<anyhow::Error> = None;

    for attempt in 1..=args.retries {
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

        if !connected {
            audit_append(&args.audit_log, &format!("{} - attempt {} - tcp unreachable", args.target_ip, attempt));
        } else {
            let target_host = &args.target_ip;
            // NOTE: we call the hex-string API. If your smbclient requires bytes, tell me y confirmo la API.
            match SmbClient::new(target_host, &args.username, &*ntlm_hash) {
                Ok(mut client) => {
                    if let Err(e) = client.connect() {
                        let err_any = anyhow::Error::new(e);
                        warn!("smb connect failed: {}", err_any);
                        audit_append(&args.audit_log, &format!("{} - attempt {} - smb connect failed: {}", args.target_ip, attempt, err_any));
                        if is_auth_error(&err_any) {
                            // auth unlikely to change → fail fast with specific code
                            if args.json {
                                let out = json!({
                                    "target": target_host,
                                    "addr": connect_addr.map(|a| a.to_string()),
                                    "error_type": "auth_failed",
                                    "message": format!("{}", err_any),
                                    "duration_ms": start.elapsed().as_millis()
                                });
                                println!("{}", out.to_string());
                            } else {
                                error!("auth failed: {:?}", err_any);
                            }
                            process::exit(4);
                        }
                        last_err = Some(err_any);
                    } else {
                        match client.list_shares() {
                            Ok(shares) => {
                                let shares_limited: Vec<String> = shares.into_iter().take(500).collect();
                                if shares_limited.is_empty() {
                                    audit_append(&args.audit_log, &format!("{} - attempt {} - no shares", args.target_ip, attempt));
                                    if args.json {
                                        let out = json!({
                                            "target": target_host,
                                            "addr": connect_addr.map(|a| a.to_string()),
                                            "error_type": "no_shares",
                                            "message": "no shares found",
                                            "duration_ms": start.elapsed().as_millis()
                                        });
                                        println!("{}", out.to_string());
                                    } else {
                                        error!("no shares found");
                                    }
                                    process::exit(5);
                                } else {
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
                                    audit_append(&args.audit_log, &format!("{} - success - {} shares - duration {}ms", args.target_ip, shares_limited.len(), start.elapsed().as_millis()));
                                    process::exit(0);
                                }
                            }
                            Err(e) => {
                                let err_any = anyhow::Error::new(e);
                                warn!("list_shares failed: {}", err_any);
                                audit_append(&args.audit_log, &format!("{} - attempt {} - list_shares failed: {}", args.target_ip, attempt, err_any));
                                last_err = Some(err_any);
                            }
                        }
                    }
                }
                Err(e) => {
                    let err_any = anyhow::Error::new(e);
                    warn!("SmbClient::new failed: {}", err_any);
                    audit_append(&args.audit_log, &format!("{} - attempt {} - client new failed: {}", args.target_ip, attempt, err_any));
                    last_err = Some(err_any);
                }
            }
        }

        // sleep backoff + optional rate-limit BETWEEN attempts (but not after last)
        if attempt < args.retries {
            thread::sleep(Duration::from_millis(args.retry_backoff_ms * attempt as u64));
            if args.rate_limit_ms > 0 {
                thread::sleep(Duration::from_millis(args.rate_limit_ms));
            }
        }
    }

    // after attempts exhausted
    if let Some(e) = last_err {
        if args.json {
            let out = json!({
                "target": args.target_ip,
                "error_type": "last_error",
                "message": format!("{:?}", e),
            });
            println!("{}", out.to_string());
        } else {
            error!("final error: {:?}", e);
        }
        process::exit(1);
    } else {
        if args.json {
            let out = json!({
                "target": args.target_ip,
                "error_type": "unreachable_or_no_shares",
                "message": "unable to reach host or no shares found"
            });
            println!("{}", out.to_string());
        } else {
            error!("unable to reach host or no shares found");
        }
        process::exit(1);
    }
}





// cargo run --release -- 192.168.0.10 administrator 8846f7eaee8fb117ad06bdd830b7586c