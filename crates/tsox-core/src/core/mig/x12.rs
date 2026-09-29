use std::any::Any;

use super::context::Context;

pub const REQUEST_ID_KEY: &str = "requestID";

pub fn context_with_value<V: Any + Send + Sync>(
    ctx: Context,
    key: &'static str,
    value: V,
) -> Context {
    ctx.with_value(key, value)
}

pub fn utf16_len(s: &str) -> usize {
    match s.find(|c: char| !c.is_ascii()) {
        None => s.len(),
        Some(i) => {
            let mut n = i;
            for r in s[i..].chars() {
                n += r.len_utf16();
            }
            n
        }
    }
}

pub fn with_request_id(ctx: Context, id: impl Into<String>) -> Context {
    context_with_value(ctx, REQUEST_ID_KEY, id.into())
}
