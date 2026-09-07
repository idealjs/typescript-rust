use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct JsonrpcVersion;

#[allow(dead_code)]
const JSON_RPC_VERSION: &str = r#""2.0""#;

impl Serialize for JsonrpcVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("2.0")
    }
}

impl<'de> Deserialize<'de> for JsonrpcVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        if s != "2.0" {
            return Err(serde::de::Error::custom("invalid JSON-RPC version"));
        }
        Ok(JsonrpcVersion)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Id {
    Int(i32),
    Str(String),
}

impl Id {
    pub fn new_string(s: &str) -> Self {
        Id::Str(s.to_string())
    }

    pub fn new_int(i: i32) -> Self {
        Id::Int(i)
    }

    pub fn as_string(&self) -> String {
        match self {
            Id::Str(s) => s.clone(),
            Id::Int(i) => i.to_string(),
        }
    }

    pub fn try_int(&self) -> Option<i32> {
        match self {
            Id::Int(i) => Some(*i),
            Id::Str(_) => None,
        }
    }

    pub fn must_int(&self) -> i32 {
        match self {
            Id::Int(i) => *i,
            Id::Str(_) => panic!("ID is not an integer"),
        }
    }
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Id::Int(i) => serializer.serialize_i32(*i),
            Id::Str(s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::String(s) => Ok(Id::Str(s)),
            Value::Number(n) => {
                let i = n
                    .as_i64()
                    .and_then(|v| i32::try_from(v).ok())
                    .ok_or_else(|| serde::de::Error::custom("ID integer out of range"))?;
                Ok(Id::Int(i))
            }
            _ => Err(serde::de::Error::custom("ID must be string or integer")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]: {}", self.code, self.message)
    }
}

impl std::error::Error for ResponseError {}

pub const CODE_PARSE_ERROR: i32 = -32700;
pub const CODE_INVALID_REQUEST: i32 = -32600;
pub const CODE_METHOD_NOT_FOUND: i32 = -32601;
pub const CODE_INVALID_PARAMS: i32 = -32602;
pub const CODE_INTERNAL_ERROR: i32 = -32603;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    Notification,
    Request,
    Response,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    #[serde(default)]
    pub jsonrpc: JsonrpcVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Id>,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

impl Message {
    pub fn kind(&self) -> MessageKind {
        if self.id.is_some() && self.method.is_empty() {
            MessageKind::Response
        } else if self.id.is_none() {
            MessageKind::Notification
        } else {
            MessageKind::Request
        }
    }

    pub fn is_request(&self) -> bool {
        self.id.is_some() && !self.method.is_empty()
    }

    pub fn is_notification(&self) -> bool {
        self.id.is_none() && !self.method.is_empty()
    }

    pub fn is_response(&self) -> bool {
        self.id.is_some() && self.method.is_empty()
    }
}
