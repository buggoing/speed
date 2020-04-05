#![feature(exclusive_range_pattern)]

use clap::{App, AppSettings, Arg};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::{thread, time, time::SystemTime};

const K_BIT: u64 = 1024;
const M_BIT: u64 = K_BIT * 1024;
const G_BIT: u64 = M_BIT * 1024;
const T_BIT: u64 = G_BIT * 1024;

const BUFF_SIZE: usize = 4096;

fn main() {
    println!("Hello, world!");
    let matches = App::new("speed")
        .about("bandwidth speed test")
        .version("0.0.1")
        .author("buggoing")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            App::new("send").about("send data").arg(
                Arg::with_name("port")
                    .short("p")
                    .default_value("2222")
                    .help("listen port")
                    .takes_value(true),
            ),
        )
        .subcommand(
            App::new("recv").about("recv data").args(&[
                Arg::with_name("server")
                    .short("s")
                    .help("server address")
                    .takes_value(true)
                    .multiple(true),
                Arg::with_name("num")
                    .short("n")
                    .default_value("1")
                    .help("connection num")
                    .takes_value(true),
            ]),
        )
        .get_matches();

    match matches.subcommand() {
        ("send", Some(send_matches)) => {
            let port = send_matches.value_of("port").unwrap();
            println!("send via port: {}", port);
            send_server(port)
        }
        ("recv", Some(recv_matches)) => {
            let servers: Vec<&str> = recv_matches.values_of("server").unwrap().collect();
            let client_num: usize = recv_matches.value_of("num").unwrap().parse().unwrap();
            recv_client(servers, client_num);
        }
        _ => println!("invalid command {}", matches.usage()),
    }
}

fn send_server(port: &str) {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(addr).unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                thread::spawn(move || {
                    // connection succeeded
                    let buf = [0 as u8; BUFF_SIZE];
                    s.write(&buf).unwrap();
                    println!("New connection: {}", s.peer_addr().unwrap());
                    loop {
                        match s.write(&buf) {
                            Ok(_) => {
                                // println!("send {} bytes", n);
                                // thread::sleep(time::Duration::from_secs(1));
                                // thread::sleep(time::Duration::from_millis(10));
                            }
                            Err(e) => {
                                println!(
                                    "failed to send data to {} err: {}",
                                    s.peer_addr().unwrap(),
                                    e
                                );
                                break;
                            }
                        }
                    }
                });
            }
            Err(e) => println!("error in listener.incoming: {}", e),
        }
    }
}

fn recv_client(servers: Vec<&str>, client_num: usize) {
    let recv_total = Arc::new(Mutex::new(0));
    for server in servers {
        for _ in 0..client_num {
            let recv_total = Arc::clone(&recv_total);
            let addr = String::from(server);
            thread::spawn(move || {
                let s = addr.clone();
                match TcpStream::connect(addr) {
                    Ok(mut stream) => {
                        println!("connect to sever: {}", s);
                        let mut buf = [0 as u8; BUFF_SIZE];
                        loop {
                            match stream.read_exact(&mut buf) {
                                Ok(_) => {
                                    // println!("read {} bytes", BUFF_SIZE);
                                    let mut recv_total = recv_total.lock().unwrap();
                                    *recv_total += BUFF_SIZE as u64;
                                }
                                Err(e) => {
                                    println!(
                                        "read failed from {} err: {}",
                                        stream.peer_addr().unwrap(),
                                        e
                                    );
                                    break;
                                }
                            }
                        }
                        // handle.join().unwrap();
                    }
                    Err(e) => println!("failed to connect server {} err: {}", s, e),
                }
            });
        }
    }
    let mut recv_last: u64 = 0;
    let mut time_last = SystemTime::now();
    loop {
        thread::sleep(time::Duration::from_secs(1));
        let recv_total = *recv_total.lock().unwrap();
        let recv_bit = (recv_total - recv_last) * 8;
        let time_now = SystemTime::now();
        let elapsed = time_now.duration_since(time_last).unwrap();
        time_last = time_now;
        recv_last = recv_total;
        let speed = match recv_bit {
            0..K_BIT => format!("{:.2} bps", recv_bit as f64 / elapsed.as_secs_f64()),
            K_BIT..M_BIT => format!(
                "{:.2} Kbps",
                (recv_bit as f64 / K_BIT as f64) / elapsed.as_secs_f64()
            ),
            M_BIT..G_BIT => format!(
                "{:.2} Mbps",
                (recv_bit as f64 / M_BIT as f64) / elapsed.as_secs_f64()
            ),
            G_BIT..T_BIT => format!(
                "{:.2} Gbps",
                (recv_bit as f64 / G_BIT as f64) / elapsed.as_secs_f64()
            ),
            _ => format!("{:.2} Tbps", recv_bit as f64 / T_BIT as f64),
        };
        println!("speed: {}", speed)
    }
}
