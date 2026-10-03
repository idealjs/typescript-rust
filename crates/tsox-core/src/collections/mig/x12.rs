use serde::de::DeserializeOwned;

use crate::collections::ordered_map::OrderedMap;
use crate::collections::ordered_set::OrderedSet;

#[derive(Debug)]
pub struct Error(String);

impl Error {
    pub fn new(message: &str) -> Self { crate::fntrace::enter("new"); 
        Error(message.to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { crate::fntrace::enter("fmt"); 
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self { crate::fntrace::enter("from"); 
        Error(e.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonKind {
    Null,
    True,
    False,
    Number,
    String,
    ArrayStart,
    ArrayEnd,
    ObjectStart,
    ObjectEnd,
}

pub struct Token {
    kind: JsonKind,
}

impl Token {
    pub fn kind(&self) -> JsonKind { crate::fntrace::enter("kind"); 
        self.kind
    }
}

pub struct Decoder<'a> {
    remaining: &'a [u8],
}

impl<'a> Decoder<'a> {
    pub fn new(data: &'a [u8]) -> Self { crate::fntrace::enter("new"); 
        Decoder { remaining: data }
    }

    pub fn peek_kind(&self) -> Result<JsonKind, Error> { crate::fntrace::enter("peek_kind"); 
        match skip_ws(self.remaining).first().copied() {
            Some(b'{') => Ok(JsonKind::ObjectStart),
            Some(b'}') => Ok(JsonKind::ObjectEnd),
            Some(b'[') => Ok(JsonKind::ArrayStart),
            Some(b']') => Ok(JsonKind::ArrayEnd),
            Some(b'"') => Ok(JsonKind::String),
            Some(b't') => Ok(JsonKind::True),
            Some(b'f') => Ok(JsonKind::False),
            Some(b'n') => Ok(JsonKind::Null),
            Some(b'-') | Some(b'0'..=b'9') => Ok(JsonKind::Number),
            Some(_) => Err(Error::new("invalid character")),
            None => Err(Error::new("unexpected end of JSON input")),
        }
    }

    pub fn read_token(&mut self) -> Result<Token, Error> { crate::fntrace::enter("read_token"); 
        let kind = self.peek_kind()?;
        match kind {
            JsonKind::Null => self.advance(4),
            JsonKind::True => self.advance(4),
            JsonKind::False => self.advance(5),
            JsonKind::ObjectStart | JsonKind::ObjectEnd | JsonKind::ArrayStart
            | JsonKind::ArrayEnd => self.advance(1),
            JsonKind::String | JsonKind::Number => {
                let _: serde_json::Value = unmarshal_decode(self)?;
            }
        }
        Ok(Token { kind })
    }

    fn advance(&mut self, n: usize) { crate::fntrace::enter("advance"); 
        self.remaining = &self.remaining[n..];
    }
}

pub fn unmarshal_decode<T: DeserializeOwned>(dec: &mut Decoder) -> Result<T, Error> { crate::fntrace::enter("unmarshal_decode"); 
    let trimmed = skip_ws(dec.remaining);
    let mut stream = serde_json::Deserializer::from_slice(trimmed).into_iter::<T>();
    let value = match stream.next() {
        Some(result) => result.map_err(Error::from)?,
        None => return Err(Error::new("unexpected end of JSON input")),
    };
    dec.remaining = &trimmed[stream.byte_offset()..];
    Ok(value)
}

fn skip_ws(data: &[u8]) -> &[u8] { crate::fntrace::enter("skip_ws"); 
    let n = data
        .iter()
        .position(|b| !b.is_ascii_whitespace())
        .unwrap_or(data.len());
    &data[n..]
}

impl<K, V> OrderedMap<K, V>
where
    K: DeserializeOwned + Eq + std::hash::Hash + Clone,
    V: DeserializeOwned + Clone,
{
    pub fn unmarshal_json_from(&mut self, dec: &mut Decoder) -> Result<(), Error> { crate::fntrace::enter("unmarshal_json_from"); 
        let token = dec.read_token()?;
        if token.kind() == JsonKind::Null {
            return Ok(());
        }
        if token.kind() != JsonKind::ObjectStart {
            return Err(Error::new("cannot unmarshal non-object JSON value into Map"));
        }
        while dec.peek_kind()? != JsonKind::ObjectEnd {
            let key: K = unmarshal_decode(dec)?;
            let value: V = unmarshal_decode(dec)?;
            self.set(key, value);
        }
        dec.read_token()?;
        Ok(())
    }
}

impl<T: Clone + Eq + std::hash::Hash> OrderedSet<T> {
    pub fn values(&self) -> impl Iterator<Item = &T> { crate::fntrace::enter("values"); 
        self.iter()
    }
}
