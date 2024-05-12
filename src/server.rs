use std::fmt;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::TcpStream;

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

    pub fn read_line (&mut self) -> anyhow::Result<String> {
        let mut line = String::new();
        self.reader.read_line(&mut line)?;
        Ok(line)
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