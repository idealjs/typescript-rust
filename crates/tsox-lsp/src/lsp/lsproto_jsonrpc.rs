use crate::jsonrpc::jsonrpc::Id as JsonrpcId;

pub fn new_id(int_val: Option<i32>, str_val: Option<&str>) -> JsonrpcId {
    if let Some(s) = str_val {
        return JsonrpcId::new_string(s);
    }
    JsonrpcId::new_int(int_val.unwrap_or(0))
}

pub use crate::jsonrpc::jsonrpc::{
    Id, JsonrpcVersion, Message as RawMessage, MessageKind, ResponseError as RawResponseError,
};
