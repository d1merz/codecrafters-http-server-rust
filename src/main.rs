mod server;

// Uncomment this block to pass the first stage
use tokio::net::{TcpListener, TcpStream};
use crate::server::{RequestHeader, ResponseHeader, Server};

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
    let header = server.read_line().await?;
    let path = parse_path(header)?;
    match path.as_str() {
        "/" => server.with(ResponseHeader::Status(200)).await.send(None).await?,
        "/user-agent" => {
            let (headers, _) = server.read_all().await?;
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
            }
            server.with(ResponseHeader::Status(404)).await.send(None).await?
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

