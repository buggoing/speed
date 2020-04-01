use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::{thread, time};

use clap::{App, AppSettings, Arg};

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
            App::new("recv").about("recv data").arg(
                Arg::with_name("server")
                    .short("s")
                    .help("server address")
                    .takes_value(true)
                    .multiple(true),
            ),
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
            recv_client(servers)
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
                    let data = [0 as u8; BUFF_SIZE];
                    s.write(&data).unwrap();
                    println!("New connection: {}", s.peer_addr().unwrap());
                    loop {
                        match s.write(&data) {
                            Ok(n) => {
                                // println!("send {} bytes", n);
                                // thread::sleep(time::Duration::from_secs(1));
                                // thread::sleep(time::Duration::from_millis(10));
                            }
                            Err(e) => {
                                println!("failed to send data err: {}", e);
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

fn recv_client(servers: Vec<&str>) {
    let mut recv_last: u64 = 9;
    let recv_total = Arc::new(Mutex::new(0));
    for server in servers {
        let recv_total = Arc::clone(&recv_total);
        match TcpStream::connect(server) {
            Ok(mut stream) => {
                println!("connect to sever: {}", server);
                let handle = thread::spawn(move || {
                    let mut buf = [0 as u8; BUFF_SIZE];
                    loop {
                        match stream.read_exact(&mut buf) {
                            Ok(_) => {
                                // println!("read {} bytes", BUFF_SIZE);
                                let mut recv_total = recv_total.lock().unwrap();
                                *recv_total += BUFF_SIZE as u64;
                                //
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
                });
                // handle.join().unwrap();
            }
            Err(e) => println!("failed to connect to {} err: {}", server, e),
        }
    }
    loop {
        thread::sleep(time::Duration::from_secs(1));
        let recv_total = *recv_total.lock().unwrap();
        let recv_number = (recv_total - recv_last) * 8;
        recv_last = recv_total;
        println!("recv_number {}", recv_number);
        let speed = match recv_number {
            0..=Kbit => format!("{} bps", recv_number),
            Kbit..=Mbit => format!("{} Kbps", recv_number / Kbit),
            Mbit..=Gbit => format!("{} Mbps", recv_number / Mbit),
            Gbit..=Tbit => format!("{} Gbps", recv_number / Gbit),
            (_) => format!("{} Tbps", recv_number / Tbit),
        };
        println!("speed: {}", speed)
    }
}

const Kbit: u64 = 1024;
const Mbit: u64 = Kbit * 1024;
const Gbit: u64 = Mbit * 1024;
const Tbit: u64 = Gbit * 1024;
