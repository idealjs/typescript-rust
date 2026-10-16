#![allow(dead_code, unused_imports, unused_variables)]

pub trait TextMarshal {
    fn marshal_text(&self) -> Result<String, String>;
}

pub fn resolve_key_name<K: ResolveKeyName>(key: &K) -> Result<String, String> { crate::fntrace::enter("resolve_key_name"); 
    key.resolve_key_name()
}

pub trait ResolveKeyName {
    fn resolve_key_name(&self) -> Result<String, String>;
}

impl ResolveKeyName for String {
    fn resolve_key_name(&self) -> Result<String, String> { crate::fntrace::enter("resolve_key_name"); 
        Ok(self.clone())
    }
}

impl ResolveKeyName for &str {
    fn resolve_key_name(&self) -> Result<String, String> { crate::fntrace::enter("resolve_key_name"); 
        Ok((*self).to_string())
    }
}

macro_rules! impl_int_key_name {
    ($($t:ty),*) => {
        $(impl ResolveKeyName for $t {
            fn resolve_key_name(&self) -> Result<String, String> { crate::fntrace::enter("resolve_key_name"); 
                Ok(self.to_string())
            }
        })*
    };
}

impl_int_key_name!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);
