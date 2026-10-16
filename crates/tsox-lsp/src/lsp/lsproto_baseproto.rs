use crate::jsonrpc::baseproto;

pub struct BaseReader<R: std::io::Read> {
    pub inner: baseproto::Reader<R>,
}

impl<R: std::io::Read> BaseReader<R> {
    pub fn new(r: R) -> Self { ::tsox_core::fntrace::enter("new"); 
        BaseReader {
            inner: baseproto::Reader::new(r),
        }
    }
}

pub struct BaseWriter<W: std::io::Write> {
    pub inner: baseproto::Writer<W>,
}

impl<W: std::io::Write> BaseWriter<W> {
    pub fn new(w: W) -> Self { ::tsox_core::fntrace::enter("new"); 
        BaseWriter {
            inner: baseproto::Writer::new(w),
        }
    }
}
