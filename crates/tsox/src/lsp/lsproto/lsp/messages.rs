use super::uri::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::jsonrpc::jsonrpc::Id as JsonrpcId;

pub const ERR_CODE_REQUEST_CANCELLED: i32 = -32800;
pub const ERR_CODE_SERVER_CANCELLED: i32 = -32802;
pub const ERR_CODE_CONTENT_MODIFIED: i32 = -32801;
pub const ERR_CODE_REQUEST_FAILED: i32 = -32803;

pub const ERR_CODE_INVALID_REQUEST: i32 = crate::jsonrpc::jsonrpc::CODE_INVALID_REQUEST;
pub const ERR_CODE_INVALID_PARAMS: i32 = crate::jsonrpc::jsonrpc::CODE_INVALID_PARAMS;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NoParams;

impl NoParams {
    pub fn is_zero(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Null;

#[derive(Debug, Clone)]
pub struct RequestInfo {
    pub method: Method,
}

impl RequestInfo {
    pub fn new_request_message(&self, id: Option<JsonrpcId>, params: Value) -> RequestMessage {
        RequestMessage {
            jsonrpc: Default::default(),
            id,
            method: self.method.clone(),
            params: Some(params),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NotificationInfo {
    pub method: Method,
}

impl NotificationInfo {
    pub fn new_notification_message(&self, params: Value) -> RequestMessage {
        RequestMessage {
            jsonrpc: Default::default(),
            id: None,
            method: self.method.clone(),
            params: Some(params),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMessage {
    #[serde(default)]
    pub jsonrpc: crate::jsonrpc::jsonrpc::JsonrpcVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<JsonrpcId>,
    pub method: Method,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl RequestMessage {
    pub fn message(&self) -> Message {
        let kind = if self.id.is_none() {
            crate::jsonrpc::jsonrpc::MessageKind::Notification
        } else {
            crate::jsonrpc::jsonrpc::MessageKind::Request
        };
        Message {
            kind,
            msg: MessageData::Request(self.clone()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    #[serde(default)]
    pub jsonrpc: crate::jsonrpc::jsonrpc::JsonrpcVersion,
    pub id: Option<JsonrpcId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<crate::jsonrpc::jsonrpc::ResponseError>,
}

impl ResponseMessage {
    pub fn message(&self) -> Message {
        Message {
            kind: crate::jsonrpc::jsonrpc::MessageKind::Response,
            msg: MessageData::Response(self.clone()),
        }
    }
}

#[derive(Debug)]
pub enum MessageData {
    Request(RequestMessage),
    Response(ResponseMessage),
}

#[derive(Debug)]
pub struct Message {
    pub kind: crate::jsonrpc::jsonrpc::MessageKind,
    pub msg: MessageData,
}

impl Message {
    pub fn as_request(&self) -> &RequestMessage {
        match &self.msg {
            MessageData::Request(r) => r,
            _ => panic!("Message is not a request"),
        }
    }

    pub fn as_response(&self) -> &ResponseMessage {
        match &self.msg {
            MessageData::Response(r) => r,
            _ => panic!("Message is not a response"),
        }
    }
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.msg {
            MessageData::Request(r) => r.serialize(serializer),
            MessageData::Response(r) => r.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawMessage {
            #[serde(default)]
            #[allow(dead_code)]
            jsonrpc: crate::jsonrpc::jsonrpc::JsonrpcVersion,
            #[serde(skip_serializing_if = "Option::is_none")]
            id: Option<JsonrpcId>,
            #[serde(default)]
            method: Method,
            #[serde(skip_serializing_if = "Option::is_none")]
            params: Option<Value>,
            #[serde(skip_serializing_if = "Option::is_none")]
            result: Option<Value>,
            #[serde(skip_serializing_if = "Option::is_none")]
            error: Option<crate::jsonrpc::jsonrpc::ResponseError>,
        }

        let raw = RawMessage::deserialize(deserializer)?;

        if raw.id.is_some() && raw.method.is_empty() {
            return Ok(Message {
                kind: crate::jsonrpc::jsonrpc::MessageKind::Response,
                msg: MessageData::Response(ResponseMessage {
                    jsonrpc: Default::default(),
                    id: raw.id,
                    result: raw.result,
                    error: raw.error,
                }),
            });
        }

        let kind = if raw.id.is_none() {
            crate::jsonrpc::jsonrpc::MessageKind::Notification
        } else {
            crate::jsonrpc::jsonrpc::MessageKind::Request
        };

        Ok(Message {
            kind,
            msg: MessageData::Request(RequestMessage {
                jsonrpc: Default::default(),
                id: raw.id,
                method: raw.method,
                params: raw.params,
            }),
        })
    }
}
