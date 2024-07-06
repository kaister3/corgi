#![allow(unused)]

use std::convert::Infallible;
use std::fmt::{Debug, format};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use hyper::body::Body;
use hyper::{Error, HeaderMap, http, Request, Response, StatusCode, Version};
use hyper::header::CONTENT_TYPE;
use serde::Serialize;
use serde_json::Value;
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

fn format_header<T>(req: &Request<T>) -> String {
    let headers = req.headers();
    headers.iter()
        .map(|(k, v)| format!("{}: {}", k, v.to_str().unwrap_or("")))
        .collect::<Vec<String>>()
        .join("\n")
}

fn format_body(req: &Request<String>) -> String {
    match req.headers().get(CONTENT_TYPE) {
        Some(content_type) => {
            match content_type.to_str() {
                Ok(ct) if ct == "application/json" => {
                    prettify_json(req.body()).unwrap()
                },
                Ok(_) => {
                    // todo: return raw string
                    let mut s = String::from("raw: ");
                    s.push_str(req.body());
                    return s;
                },
                Err(_) => {
                    // deal with error
                    "Some error occurred".to_string()
                }
            }

        },
        None => {
            "No content type is provided".to_string()
        }
    }
}

fn prettify_json(json_str: &str) -> serde_json::Result<String> {
    let value: Value = serde_json::from_str(json_str)?;
    serde_json::to_string_pretty(&value)
}

#[cfg(test)]
mod tests {
    use hyper::{HeaderMap, Request, Version};
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