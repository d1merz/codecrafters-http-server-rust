use std::collections::HashMap;
use std::fmt;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::TcpStream;
use std::str::FromStr;

pub struct Server {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>
}

impl Server {
    pub fn new(_stream: TcpStream) -> Self {
        let stream = _stream.try_clone().expect("Cannot clone a stream");
        let reader = BufReader::new(_stream);
        let writer = BufWriter::new(stream);
        Self {reader, writer}
    }

    pub fn with(&mut self, header: ResponseHeader) -> &mut Self {
        self.writer.write_all(header.to_string().as_bytes()).expect("Cannot write to buf");
        self
    }

    pub fn send(&mut self, body: Option<String>) -> anyhow::Result<()> {
        self.writer.write_all("\r\n".as_bytes())?;
        if let Some(body) = body {
            self.writer.write_all(body.as_bytes())?;
        }
        self.writer.flush()?;
        Ok(())
    }

    pub fn read_line(&mut self) -> anyhow::Result<String> {
        let mut line = String::new();
        self.reader.read_line(&mut line)?;
        Ok(line)
    }

    pub fn read_all(&mut self) -> anyhow::Result<(HashMap<RequestHeader, String>, String)> {
        let mut headers= HashMap::new();
        loop {
            if let Ok(line) = self.read_line() {
                if line == "\r\n" { break }
                let tokens: Vec<String> = line.split(": ").map(|s| s.to_string()).collect();
                if let Some(value) = tokens.get(1)  {
                    if let Ok(header) = RequestHeader::from_str(tokens.get(0).unwrap()) {
                        headers.insert(header, value.trim().to_string());
                    }
                }
            }
        }
        let body = String::new();
        Ok((headers, body))
    }
}

#[derive(Eq, PartialEq, Hash)]
pub enum RequestHeader {
    Host,
    Agent,
    Accept
}

impl FromStr for RequestHeader {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Логика для преобразования строки в значение перечисления
        match s {
            "Host" => Ok(Self::Host),
            "User-Agent" => Ok(Self::Agent),
            "Accept" => Ok(Self::Accept),
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