mod server;

// Uncomment this block to pass the first stage
use std::net::{TcpListener, TcpStream};
use crate::server::{ResponseHeader, Server};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                handle_request(_stream).expect("Cannot handle a request");
                println!("accepted new connection");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_request(_stream: TcpStream) -> anyhow::Result<()> {
    let mut server = Server::new(_stream);
    let header = server.read_line()?;
    let path = parse_path(header)?;
    match path.as_str() {
        "/" => server.with(ResponseHeader::Status(200)).send(None)?,
        _ => {
            if path.starts_with("/echo") {
                let value = &path["/echo/".len()..];
                server.with(ResponseHeader::Status(200)).
                    with(ResponseHeader::Type("text/plain".to_string())).
                    with(ResponseHeader::Length(value.len() as i32)).
                    send(Some(value.to_string()))?
            }
            server.with(ResponseHeader::Status(404)).send(None)?
        }
    }
    Ok(())
}

fn parse_path(header: String) -> anyhow::Result<String> {
    let tokens: Vec<String> = header.split(" ").map(|s| s.to_string()).collect();
    if let Some (path) = tokens.get(1) {
        return Ok(path.to_string())
    } else {
        Err(anyhow::format_err!("Invalid header format"))
    }
}

