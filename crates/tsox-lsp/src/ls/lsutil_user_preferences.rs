pub use crate::ls::lsutil_user_preferences_enums::*;
pub use crate::ls::lsutil_user_preferences_preferences::*;

/// Go fourslash SetPreference / LSP 配置：按原始键名设置单个用户偏好
pub fn set_user_preference_raw(prefs: &mut UserPreferences, raw_name: &str, value: &str) {
    let v = serde_json::Value::String(value.to_string());
    crate::ls::lsutil_user_preferences_raw_fields::apply_raw_field(prefs, raw_name, &v);
}
