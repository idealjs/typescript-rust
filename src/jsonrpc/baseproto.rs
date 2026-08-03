//! Base protocol for JSON-RPC with Content-Length headers (as used by LSP).
//!
//! Ported from Go's `internal/jsonrpc/baseproto.go`.

use std::io::{self, BufRead, BufReader, Read, Write};

// !!! Errors
// Mirrors Go's `ErrInvalidHeader`, `ErrInvalidContentLength`, `ErrNoContentLength`.

/// Reader reads JSON-RPC messages with Content-Length framing.
pub struct Reader<R: Read> {
    r: BufReader<R>,
}

impl<R: Read> Reader<R> {
    /// Creates a new Reader.
    pub fn new(r: R) -> Self {
        Reader {
            r: BufReader::new(r),
        }
    }

    /// Reads the next message payload.
    pub fn read(&mut self) -> io::Result<Vec<u8>> {
        let mut content_length: i64 = 0;

        loop {
            let mut line = Vec::new();
            self.r.read_until(b'\n', &mut line)?;

            if line == b"\r\n" {
                break;
            }

            // Parse "Key: Value" header.
            let colon = match line.iter().position(|&b| b == b':') {
                Some(idx) => idx,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "jsonrpc: invalid header: {:?}",
                            String::from_utf8_lossy(&line)
                        ),
                    ));
                }
            };

            let key = &line[..colon];
            let value = &line[colon + 1..];

            if key == b"Content-Length" {
                let trimmed_str = String::from_utf8_lossy(value);
                let trimmed = trimmed_str.trim();
                content_length = trimmed.parse::<i64>().map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("jsonrpc: invalid content length: parse error: {e}"),
                    )
                })?;
                if content_length < 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("jsonrpc: invalid content length: negative value {content_length}"),
                    ));
                }
            }
        }

        if content_length <= 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "jsonrpc: no content length",
            ));
        }

        let mut data = vec![0u8; content_length as usize];
        self.r.read_exact(&mut data)?;

        Ok(data)
    }
}

/// Writer writes JSON-RPC messages with Content-Length framing.
pub struct Writer<W: Write> {
    w: W,
}

impl<W: Write> Writer<W> {
    /// Creates a new Writer.
    pub fn new(w: W) -> Self {
        Writer { w }
    }

    /// Writes a message payload with Content-Length header.
    pub fn write(&mut self, data: &[u8]) -> io::Result<()> {
        write!(self.w, "Content-Length: {}\r\n\r\n", data.len())?;
        self.w.write_all(data)?;
        self.w.flush()
    }
}
