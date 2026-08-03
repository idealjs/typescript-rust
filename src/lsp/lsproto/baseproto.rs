//! Base protocol reader/writer for LSP (wraps jsonrpc::baseproto).
//!
//! Ported from Go's `internal/lsp/lsproto/baseproto.go`.

use crate::jsonrpc::baseproto;

/// BaseReader wraps jsonrpc::baseproto::Reader for backwards compatibility.
pub struct BaseReader<R: std::io::Read> {
    pub inner: baseproto::Reader<R>,
}

impl<R: std::io::Read> BaseReader<R> {
    /// Creates a new BaseReader.
    pub fn new(r: R) -> Self {
        BaseReader {
            inner: baseproto::Reader::new(r),
        }
    }
}

/// BaseWriter wraps jsonrpc::baseproto::Writer for backwards compatibility.
pub struct BaseWriter<W: std::io::Write> {
    pub inner: baseproto::Writer<W>,
}

impl<W: std::io::Write> BaseWriter<W> {
    /// Creates a new BaseWriter.
    pub fn new(w: W) -> Self {
        BaseWriter {
            inner: baseproto::Writer::new(w),
        }
    }
}
