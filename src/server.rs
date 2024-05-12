use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

pub struct Server {
    reader: BufReader<OwnedReadHalf>,
    writer: BufWriter<OwnedWriteHalf>
}

impl Server {
    pub fn new(_stream: TcpStream) -> Self {
        let (read, write) = _stream.into_split();
        let reader = BufReader::new(read);
        let writer = BufWriter::new(write);
        Self {reader, writer}
    }

    pub async fn with(&mut self, header: ResponseHeader) -> &mut Self {
        self.writer.write_all(header.to_string().as_bytes()).await.expect("Cannot write a header");
        self
    }

    pub async fn send(&mut self, body: Option<String>) -> anyhow::Result<()> {
        self.writer.write_all("\r\n".as_bytes()).await?;
        if let Some(body) = body {
            self.writer.write_all(body.as_bytes()).await?;
        }
        self.writer.flush().await?;
        Ok(())
    }

    async fn read_line(&mut self) -> anyhow::Result<String> {
        let mut line = String::new();
        self.reader.read_line(&mut line).await?;
        Ok(line)
    }

    pub async fn read_all(&mut self) -> anyhow::Result<(HttpMethod, String, HashMap<RequestHeader, String>, String)> {
        let header = self.read_line().await?;
        let (method, path) = parse_header(header)?;
        let mut headers= HashMap::new();
        loop {
            if let Ok(line) = self.read_line().await {
                if line == "\r\n" { break }
                let tokens: Vec<String> = line.split(": ").map(|s| s.to_string()).collect();
                if let Some(value) = tokens.get(1)  {
                    if let Ok(header) = RequestHeader::from_str(tokens.get(0).unwrap()) {
                        headers.insert(header, value.trim().to_string());
                    }
                }
            }
        }
        let mut body = String::new();
        if let Some(len) = headers.get(&RequestHeader::Length) {
            let length: i32 = len.parse()?;
            let mut buf = vec![0u8; length as usize];
            self.reader.read_exact(&mut buf).await?;
            body = String::from_utf8_lossy(&buf).to_string();
        }
        Ok((method, path, headers, body))
    }
}

#[derive(Eq, PartialEq, Hash)]
pub enum RequestHeader {
    Host,
    Agent,
    Accept,
    Length
}

impl FromStr for RequestHeader {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Логика для преобразования строки в значение перечисления
        match s {
            "Host" => Ok(Self::Host),
            "User-Agent" => Ok(Self::Agent),
            "Accept" => Ok(Self::Accept),
            "Content-Length" => Ok(Self::Length),
            _ => Err("Неверный формат строки".to_string()),
        }
    }
}



pub enum ResponseHeader {
    Status(i32),
    Type(String),
    Length(i32)
}

impl fmt::Display for ResponseHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Здесь должна быть логика форматирования для каждого варианта
        match self {
            Self::Status(code) => {
                match code {
                    201 => write!(f, "HTTP/1.1 {code} Created\r\n"),
                    200..=299 => write!(f, "HTTP/1.1 {code} OK\r\n"),
                    400..=499 => write!(f, "HTTP/1.1 {code} Not Found\r\n"),
                    _ => {Ok(())}
                }
            },
            Self::Type(content_type) => write!(f, "Content-Type: {content_type}\r\n"),
            Self::Length(len) => write!(f, "Content-Length: {len}\r\n")
        }
    }
}

pub enum HttpMethod {
    Get,
    Post
}

impl FromStr for HttpMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Логика для преобразования строки в значение перечисления
        match s {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            _ => Err("Неверный формат строки".to_string()),
        }
    }
}

fn parse_header(header: String) -> anyhow::Result<(HttpMethod, String)> {
    let tokens: Vec<String> = header.split(" ").map(|s| s.to_string()).collect();
    if let Some(path) = tokens.get(1) {
        let method = HttpMethod::from_str(tokens.get(0).unwrap()).unwrap();
        Ok((method, path.to_string()))
    } else {
        Err(anyhow::format_err!("Invalid header format"))
    }
}