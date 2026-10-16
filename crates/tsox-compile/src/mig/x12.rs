use serde::de::DeserializeOwned;
use tsox_core::collections::mig::x12::{unmarshal_decode, Decoder, Error};
use tsox_core::json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DiagnosticDirectivePolicy(pub u8);

impl DiagnosticDirectivePolicy {
    pub const IGNORE: DiagnosticDirectivePolicy = DiagnosticDirectivePolicy(0);
    pub const EXPECT: DiagnosticDirectivePolicy = DiagnosticDirectivePolicy(1);
}

#[derive(Debug, Clone, Default)]
pub struct MappedDiagnosticDirective {
    pub original_start: i32,
    pub original_length: i32,
    pub virtual_start: i32,
    pub virtual_end: i32,
    pub policy: DiagnosticDirectivePolicy,
    pub unused_expect_directive_index: Option<i32>,
}

fn json_unmarshal<T: DeserializeOwned>(value: &Value, out: &mut T) -> Result<(), Error> { ::tsox_core::fntrace::enter("json_unmarshal"); 
    <T as serde::Deserialize>::deserialize(value)
        .map(|_: T| ())
        .map_err(|e| Error::new(&e.to_string()))
}

impl MappedDiagnosticDirective {
    pub fn unmarshal_json_from(&mut self, dec: &mut Decoder) -> Result<(), Error> { ::tsox_core::fntrace::enter("unmarshal_json_from"); 
        let tuple: Vec<Value> = unmarshal_decode(dec)?;
        if tuple.len() != 5 && tuple.len() != 6 {
            return Err(Error::new(&format!(
                "diagnostic directive tuple must contain 5 or 6 elements, got {}",
                tuple.len()
            )));
        }
        *self = MappedDiagnosticDirective::default();
        json_unmarshal(&tuple[0], &mut self.original_start)?;
        json_unmarshal(&tuple[1], &mut self.original_length)?;
        json_unmarshal(&tuple[2], &mut self.virtual_start)?;
        json_unmarshal(&tuple[3], &mut self.virtual_end)?;
        let mut policy = 0u8;
        json_unmarshal(&tuple[4], &mut policy)?;
        self.policy = DiagnosticDirectivePolicy(policy);
        if tuple.len() == 6 {
            let mut index = 0;
            json_unmarshal(&tuple[5], &mut index)?;
            self.unused_expect_directive_index = Some(index);
        }
        Ok(())
    }
}
