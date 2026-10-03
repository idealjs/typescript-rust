use crate::packagejson::json::{JsonValue, JsonValueType};
use tsox_core::collections::mig::x12::{unmarshal_decode, Decoder, Error, JsonKind};

pub fn unmarshal_json_value(v: &mut JsonValue, data: &[u8]) -> Result<(), Error> { ::tsox_core::fntrace::enter("unmarshal_json_value"); 
    if data == b"null" {
        *v = JsonValue {
            value_type: JsonValueType::Null,
            ..Default::default()
        };
        return Ok(());
    }
    match data.first().copied().unwrap_or(b' ') {
        b'"' => {
            let s: String = serde_json::from_slice(data).map_err(Error::from)?;
            v.value_type = JsonValueType::String;
            v.string_value = Some(s);
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
        _ if data == b"true" => {
            v.value_type = JsonValueType::Boolean;
            v.bool_value = Some(true);
        }
        _ if data == b"false" => {
            v.value_type = JsonValueType::Boolean;
            v.bool_value = Some(false);
        }
        _ => {
            let n: f64 = serde_json::from_slice(data).map_err(Error::from)?;
            v.value_type = JsonValueType::Number;
            v.number_value = Some(n);
        }
    }
    Ok(())
}

pub fn unmarshal_json_value_v2(v: &mut JsonValue, dec: &mut Decoder) -> Result<(), Error> { ::tsox_core::fntrace::enter("unmarshal_json_value_v2"); 
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
