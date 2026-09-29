use serde::de::DeserializeOwned;

use crate::packagejson::json::{JsonValueType, JsonValue};
use tsox_core::collections::mig::x12::{unmarshal_decode, Decoder, Error, JsonKind};

pub fn unmarshal_json_value<T: DeserializeOwned>(
    v: &mut JsonValue,
    data: &[u8],
) -> Result<(), Error> {
    if data == b"null" {
        *v = JsonValue {
            value_type: JsonValueType::Null,
            ..Default::default()
        };
        return Ok(());
    }
    match data[0] {
        b'"' => {
            v.value_type = JsonValueType::String;
            v.string_value = Some(serde_json::from_slice(data).map_err(Error::from)?);
        }
        b'[' => {
            let elements: Vec<serde_json::Value> = serde_json::from_slice(data).map_err(Error::from)?;
            v.value_type = JsonValueType::Array;
            v.array_value = Some(elements.into_iter().map(JsonValue::from).collect());
        }
        b'{' => {
            let object: serde_json::Map<String, serde_json::Value> =
                serde_json::from_slice(data).map_err(Error::from)?;
            v.value_type = JsonValueType::Object;
            v.object_value = Some(
                object
                    .into_iter()
                    .map(|(key, value)| (key, JsonValue::from(value)))
                    .collect(),
            );
        }
        _ if data == b"true" || data == b"false" => {
            v.value_type = JsonValueType::Boolean;
            v.bool_value = Some(data == b"true");
        }
        _ => {
            v.value_type = JsonValueType::Number;
            v.number_value = Some(serde_json::from_slice(data).map_err(Error::from)?);
        }
    }
    Ok(())
}

pub fn unmarshal_json_value_v2<T: DeserializeOwned>(
    v: &mut JsonValue,
    dec: &mut Decoder,
) -> Result<(), Error> {
    match dec.peek_kind()? {
        JsonKind::Null => {
            dec.read_token()?;
            *v = JsonValue {
                value_type: JsonValueType::Null,
                ..Default::default()
            };
        }
        JsonKind::String => {
            v.value_type = JsonValueType::String;
            v.string_value = Some(unmarshal_decode(dec)?);
        }
        JsonKind::ArrayStart => {
            dec.read_token()?;
            let mut elements: Vec<JsonValue> = Vec::new();
            while dec.peek_kind()? != JsonKind::ArrayEnd {
                let element: serde_json::Value = unmarshal_decode(dec)?;
                elements.push(JsonValue::from(element));
            }
            dec.read_token()?;
            v.value_type = JsonValueType::Array;
            v.array_value = Some(elements);
        }
        JsonKind::ObjectStart => {
            let object: serde_json::Map<String, serde_json::Value> = unmarshal_decode(dec)?;
            v.value_type = JsonValueType::Object;
            v.object_value = Some(
                object
                    .into_iter()
                    .map(|(key, value)| (key, JsonValue::from(value)))
                    .collect(),
            );
        }
        JsonKind::True | JsonKind::False => {
            v.value_type = JsonValueType::Boolean;
            v.bool_value = Some(unmarshal_decode(dec)?);
        }
        JsonKind::Number => {
            v.value_type = JsonValueType::Number;
            v.number_value = Some(unmarshal_decode(dec)?);
        }
        _ => {}
    }
    Ok(())
}

#[derive(Clone, Debug, Default)]
pub struct InfoCacheEntry {
    pub package_directory: String,
    pub directory_exists: bool,
    pub contents: Option<crate::packagejson::Fields>,
}

impl InfoCacheEntry {
    pub fn exists(&self) -> bool {
        self.contents.is_some()
    }

    pub fn get_contents(&self) -> Option<&crate::packagejson::Fields> {
        self.contents.as_ref()
    }

    pub fn get_directory(&self) -> &str {
        &self.package_directory
    }

    pub fn with_package_directory(&self, package_directory: impl Into<String>) -> Self {
        let package_directory = package_directory.into();
        if self.package_directory != package_directory {
            let mut copy = self.clone();
            copy.package_directory = package_directory;
            copy
        } else {
            self.clone()
        }
    }
}
