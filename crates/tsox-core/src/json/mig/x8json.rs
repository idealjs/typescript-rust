#![allow(dead_code, unused_imports, unused_variables)]

use std::io::Write;

pub fn marshal_encode<W: Write, T: serde::Serialize>(out: &mut W, value: &T) -> Result<(), serde_json::Error> { crate::fntrace::enter("marshal_encode"); 
    serde_json::to_writer(out, value)
}
