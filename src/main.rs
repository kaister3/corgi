#![allow(unused)]
#![deny(warnings)]

use std::convert::Infallible;
use std::fmt::Debug;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::ops::Deref;
use http_body_util::{BodyExt, Full};
use hyper::{http, Request, Response, StatusCode, Version};
use hyper::body::{Body, Bytes};
use hyper::header::CONTENT_TYPE;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::{TokioIo, TokioTimer};
use serde::Serialize;
use serde_json::Value;
use structopt::StructOpt;
use tokio::net::TcpListener;

#[derive(Debug, StructOpt)]
#[structopt(name = "Corgi")]
struct Corgi {
    #[structopt(short = "p", long = "port", default_value = "8080")]
    port: u16,
    #[structopt(long)]
    pretty: bool
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listening_port = Corgi::from_args().port;
    let addr: SocketAddr = ([127, 0, 0, 1], listening_port).into();

    let listener = TcpListener::bind(addr).await?;
    println!("Corgi HTTP request logger is listening on port {}", listening_port);
    println!("Version: {}", env!("CARGO_PKG_VERSION"));

    loop {
        let (tcp, _) = listener.accept().await?;
        let io = TokioIo::new(tcp);

        tokio::task::spawn(async move {
            // todo add http2 support with hyper-util
            if let Err(err) = http1::Builder::new()
                .timer(TokioTimer::new())
                .serve_connection(io, service_fn(handle_req_v2))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn handle_req_v2(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    println!("{} {} {:?}", req.method(), req.uri(), req.version());
    println!("headers: ");
    let header_string = req.headers().iter().map(|(k, v)| {
        format!("{} {:?}", k, v)
    }).collect::<Vec<String>>()
    .join("\n");
    println!("{}", header_string);
    println!("body: ");
    // req.into_body()

    Ok(Response::new(Full::new(Bytes::from("woof woof"))))
}

fn format_header<T>(req: &Request<T>) -> String {
    let headers = req.headers();
    headers.iter()
        .map(|(k, v)| format!("{}: {}", k, v.to_str().unwrap_or("")))
        .collect::<Vec<String>>()
        .join("\n")
}