#![allow(unused)]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "Corgi")]
struct Corgi {
    #[structopt(short = "p", long = "port", default_value = "8080")]
    port: u16,
    #[structopt(long)]
    pretty: bool
}

fn main() -> std::io::Result<()> {
    let listening_port = Corgi::from_args().port.to_string();
    let addr = format!("127.0.0.1:{}", listening_port);
    let listener = TcpListener::bind(addr)?;
    println!("Corgi HTTP request logger is listening on port {}", listening_port);
    println!("Version: {}", env!("CARGO_PKG_VERSION"));

    for stream in listener.incoming() {
        let mut stream = stream?;
        handle_client(&mut stream)?;
    }

    Ok(())
}

fn handle_client(mut tcp_stream: &mut TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 512];
    tcp_stream.read(&mut buffer)?;

    let request = String::from_utf8_lossy(&buffer[..]);
    println!("{}", request);

    let response = "HTTP/1.1 200 OK\r\nContent-Length: 9\r\n\r\nwoof woof";
    tcp_stream.write(response.as_bytes())?;
    tcp_stream.flush()?;

    Ok(())
}