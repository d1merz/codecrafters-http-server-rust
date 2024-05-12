use std::io::{BufRead, BufReader, BufWriter, Write};
// Uncomment this block to pass the first stage
use std::net::{TcpListener, TcpStream};

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
    let stream = _stream.try_clone()?;
    let mut reader = BufReader::new(_stream);
    let mut writer = BufWriter::new(stream);
    let mut header = String::new();
    reader.read_line(&mut header)?;
    let path = parse_path(header)?;
    match path.as_str() {
        "/" => writer.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).unwrap(),
        _ => writer.write_all("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes()).unwrap()
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

