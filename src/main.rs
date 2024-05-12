mod server;

use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
// Uncomment this block to pass the first stage
use tokio::net::{TcpListener, TcpStream};
use crate::server::{HttpMethod, RequestHeader, ResponseHeader, Server};

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            let _ = handle_request(stream).await;
        });
    }
}

async fn handle_request(_stream: TcpStream) -> anyhow::Result<()> {
    let mut server = Server::new(_stream);
    let (method, path, headers, body) = server.read_all().await?;
    match path.as_str() {
        "/" => server.with(ResponseHeader::Status(200)).await.send(None).await?,
        "/user-agent" => {
            if let Some(value) = headers.get(&RequestHeader::Agent) {
                server.with(ResponseHeader::Status(200)).await.
                    with(ResponseHeader::Type("text/plain".to_string())).await.
                    with(ResponseHeader::Length(value.len() as i32)).await.
                    send(Some(value.to_string())).await?
            } else {
                return Err(anyhow::format_err!("Cannot parse a user/agent"));
            }
        }
        _ => {
            if path.starts_with("/echo") {
                let value = &path["/echo/".len()..];
                server.with(ResponseHeader::Status(200)).await.
                    with(ResponseHeader::Type("text/plain".to_string())).await.
                    with(ResponseHeader::Length(value.len() as i32)).await.
                    send(Some(value.to_string())).await?
            } else if path.starts_with("/files") {
                let filename = &path["/files/".len()..];
                let directory = std::env::args().nth(2).expect("Not enough args");
                let file_path = Path::new(&directory).join(filename);
                match method {
                    HttpMethod::Get => {
                        if let Ok(mut file) = File::open(file_path).await {
                            let mut body = String::new();
                            file.read_to_string(&mut body).await?;
                            server.with(ResponseHeader::Status(200)).await.
                                with(ResponseHeader::Type("application/octet-stream".to_string())).await.
                                with(ResponseHeader::Length(body.len() as i32)).await.
                                send(Some(body)).await?
                        } else {
                            server.with(ResponseHeader::Status(404)).await.send(None).await?
                        }
                    },
                    HttpMethod::Post => {
                        if let Ok(mut file) = File::create(file_path).await {
                            server.with(ResponseHeader::Status(201)).await.send(None).await?;
                            file.write_all(body.as_bytes()).await?;
                        } else {
                            server.with(ResponseHeader::Status(404)).await.send(None).await?
                        }
                    }
                }
            }
            else {
                server.with(ResponseHeader::Status(404)).await.send(None).await?
            }
        }
    }
    Ok(())
}
