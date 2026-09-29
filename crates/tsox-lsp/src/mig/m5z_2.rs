#![allow(dead_code, unused_imports, unused_variables)]

use std::io::{Read, Write};
use std::path::Path;

use super::m5z::JsonRpcProtocol;

pub enum Conn {
    Stdio(StdioConn),
    Unix(std::os::unix::net::UnixStream),
}

pub struct StdioConn {
    stdin: std::io::Stdin,
    stdout: std::io::Stdout,
}

impl Clone for StdioConn {
    fn clone(&self) -> Self {
        StdioConn {
            stdin: std::io::stdin(),
            stdout: std::io::stdout(),
        }
    }
}

pub trait ReadWriteCloser: Read + Write {
    fn close(&mut self) -> Result<(), String>;
}

impl Read for StdioConn {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stdin.read(buf)
    }
}

impl Write for StdioConn {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stdout.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stdout.flush()
    }
}

impl ReadWriteCloser for StdioConn {
    fn close(&mut self) -> Result<(), String> {
        let _ = self.stdin.read_to_string(&mut String::new());
        Ok(())
    }
}

impl ReadWriteCloser for std::os::unix::net::UnixStream {
    fn close(&mut self) -> Result<(), String> {
        std::os::unix::net::UnixStream::shutdown(self, std::net::Shutdown::Both)
            .map_err(|e| e.to_string())
    }
}

impl std::io::Read for Conn {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Conn::Stdio(c) => c.read(buf),
            Conn::Unix(s) => s.read(buf),
        }
    }
}

impl std::io::Write for Conn {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Conn::Stdio(c) => c.write(buf),
            Conn::Unix(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Conn::Stdio(c) => c.flush(),
            Conn::Unix(s) => s.flush(),
        }
    }
}

impl tsox_compile::mig::m3l_cm_2::ReadWriteCloser for Conn {
    fn close(&mut self) -> std::io::Result<()> {
        match self {
            Conn::Stdio(c) => ReadWriteCloser::close(c).map_err(std::io::Error::other),
            Conn::Unix(s) => std::os::unix::net::UnixStream::shutdown(s, std::net::Shutdown::Both),
        }
    }
}

pub struct PipeTransport {
    listener: std::os::unix::net::UnixListener,
    path: String,
}

pub fn new_pipe_transport(path: &str) -> Result<PipeTransport, String> {
    Ok(PipeTransport {
        listener: new_pipe_listener(path)?,
        path: path.to_string(),
    })
}

impl PipeTransport {
    pub fn accept(&self) -> Result<Conn, String> {
        let (stream, _) = self.listener.accept().map_err(|e| e.to_string())?;
        Ok(Conn::Unix(stream))
    }

    pub fn close(&self) -> Result<(), String> {
        let _ = std::fs::remove_file(&self.path);
        Ok(())
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

pub fn new_pipe_listener(path: &str) -> Result<std::os::unix::net::UnixListener, String> {
    let _ = std::fs::remove_file(path);
    std::os::unix::net::UnixListener::bind(path).map_err(|e| e.to_string())
}

pub fn generate_pipe_path(name: &str) -> String {
    std::env::temp_dir().join(name).to_string_lossy().to_string()
}

pub struct StdioTransport {
    used: bool,
}

pub fn new_stdio_transport() -> StdioTransport {
    StdioTransport { used: false }
}

impl StdioTransport {
    pub fn accept(&mut self) -> Result<Conn, String> {
        if self.used {
            return Err("EOF".to_string());
        }
        self.used = true;
        Ok(Conn::Stdio(StdioConn {
            stdin: std::io::stdin(),
            stdout: std::io::stdout(),
        }))
    }

    pub fn close(&self) -> Result<(), String> {
        Ok(())
    }
}
