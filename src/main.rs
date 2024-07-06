#![allow(unused)]
#![deny(warnings)]

use std::convert::Infallible;
use std::fmt::Debug;
use std::io::{Read, Write};
use std::net::SocketAddr;
use http_body_util::Full;
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

    let make_svc = service_fn(handle_req);

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

// fn handle_client(mut tcp_stream: &mut TcpStream) -> std::io::Result<()> {
//     let mut buffer = [0; 512];
//     tcp_stream.read(&mut buffer)?;
//
//     let request = String::from_utf8_lossy(&buffer[..]);
//     println!("{}", request);
//
//     let response = "HTTP/1.1 200 OK\r\nContent-Length: 9\r\n\r\nwoof woof";
//     tcp_stream.write(response.as_bytes())?;
//     tcp_stream.flush()?;
//
//     Ok(())
// }

async fn handle_req(req: Request<String>) -> http::Result<Response<String>> {

    let (parts, body) = req.clone().into_parts();
    let req_header = &parts.headers;
    let body_str = format_body(&req);

    match parts.version {
        Version::HTTP_11 => {
            let mut req_str = format!("{} {} {:?}", parts.method, parts.uri, parts.version);
            // format headers
            req_str = req_str + &format_header(&req);
            // format json body
            req_str = req_str + &format_body(&req);
        },
        Version::HTTP_2 => {
            todo!("handle http 2 request")
        },
        Version::HTTP_3 => {
            todo!("handle http 3 request")
        },
        _ => {
            todo!("handle older version http request")
        }
    }

    let resp = Response::builder()
        .version(Version::HTTP_11)
        .status(StatusCode::OK)
        .body("woof woof".to_string());

    return resp;
}

async fn handle_req_v2(req: Request<impl Body>) -> Result<Response<Full<Bytes>>, Infallible> {

    let mut req_str = format!("{} {} {:?}", req.method(), req.uri(), req.version());
    // format headers
    req_str = req_str + &format_header(&req);
    // format json body
    // req_str = req_str + &format_body(&req);
    req_str = req_str + req.into
    println!("{}", req_str);

    Ok(Response::new(Full::new(Bytes::from("woof woof"))))
}

fn format_header<T>(req: &Request<T>) -> String {
    let headers = req.headers();
    headers.iter()
        .map(|(k, v)| format!("{}: {}", k, v.to_str().unwrap_or("")))
        .collect::<Vec<String>>()
        .join("\n")
}

// fn format_body(req: &Request<impl Body>) -> String {
//     match req.headers().get(CONTENT_TYPE) {
//         Some(content_type) => {
//             match content_type.to_str() {
//                 Ok(ct) if ct == "application/json" => {
//                     // prettify_json(&req).unwrap()
//                     req.
//                 },
//                 Ok(_) => {
//                     // todo: return raw string
//                     let mut s = String::from("raw: ");
//                     s.push_str(req.body());
//                     return s;
//                 },
//                 Err(_) => {
//                     // deal with error
//                     "Some error occurred".to_string()
//                 }
//             }
//
//         },
//         None => {
//             "No content type is provided".to_string()
//         }
//     }
// }

// fn prettify_json(req: Request<impl Body>) -> serde_json::Result<String> {
//     let whole_body = hyper::body::to
//     let json_str = String::from_utf8_lossy(&body).to_string();
//     let value: Value = serde_json::from_str(&json_str)?;
//     serde_json::to_string_pretty(&value)
// }

#[cfg(test)]
mod tests {
    use hyper::{Request, Version};
    use serde::{Deserialize, Serialize};

    use crate::{format_body, format_header};

    #[derive(Serialize, Deserialize)]
    struct UserInfo {
        id: u64,
        name: String,
        age: u8,
        nickname: String,
    }

    #[test]
    fn test_format_header() {
        let req = Request::builder()
            .version(Version::HTTP_11)
            .header("test", "success")
            .header("key", "val")
            .body("adasdasacascas").unwrap();

        let ret = format_header(&req);
        println!("{}", ret);
    }

    #[test]
    fn test_format_body_raw() {
        let req = Request::builder()
            .version(Version::HTTP_11)
            .header("Content-Type", "text/plain")
            .body("adasdasacascas".to_string()).unwrap();


        let ret = format_body(&req);
        println!("{}", ret);
    }

    #[test]
    fn test_format_body_json() {

        let user_info = UserInfo {
            name: "Kris".to_string(),
            age: 30,
            id: 10010,
            nickname: "asdasd".to_string(),
        };

        let user_info_json = serde_json::to_string_pretty(&user_info).unwrap();

        let req = Request::builder()
            .version(Version::HTTP_11)
            .header("Content-Type", "application/json")
            .body(user_info_json).unwrap();

        let ret = format_body(&req);
        println!("{}", ret);
    }
}