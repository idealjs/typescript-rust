use std::collections::HashMap;
use std::sync::OnceLock;

use crate::locale::Locale;

pub mod messages_generated;

static LOCALE_MAPS: OnceLock<HashMap<&'static str, HashMap<String, String>>> = OnceLock::new();

fn decompress_gzip(data: &[u8]) -> Vec<u8> {
    use flate2::read::GzDecoder;
    use std::io::Read;
    let mut decoder = GzDecoder::new(data);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result).unwrap_or_default();
    result
}

fn load_locale_map() -> HashMap<&'static str, HashMap<String, String>> {
    let mut maps = HashMap::new();
    for (tag, raw) in [
        ("cs-CZ", include_bytes!("loc/cs-CZ.json.gz").as_slice()),
        ("de-DE", include_bytes!("loc/de-DE.json.gz").as_slice()),
        ("es-ES", include_bytes!("loc/es-ES.json.gz").as_slice()),
        ("fr-FR", include_bytes!("loc/fr-FR.json.gz").as_slice()),
        ("it-IT", include_bytes!("loc/it-IT.json.gz").as_slice()),
        ("ja-JP", include_bytes!("loc/ja-JP.json.gz").as_slice()),
        ("ko-KR", include_bytes!("loc/ko-KR.json.gz").as_slice()),
        ("pl-PL", include_bytes!("loc/pl-PL.json.gz").as_slice()),
        ("pt-BR", include_bytes!("loc/pt-BR.json.gz").as_slice()),
        ("ru-RU", include_bytes!("loc/ru-RU.json.gz").as_slice()),
        ("tr-TR", include_bytes!("loc/tr-TR.json.gz").as_slice()),
        ("zh-CN", include_bytes!("loc/zh-CN.json.gz").as_slice()),
        ("zh-TW", include_bytes!("loc/zh-TW.json.gz").as_slice()),
    ] {
        let decompressed = decompress_gzip(raw);
        let map: HashMap<String, String> =
            serde_json::from_slice(&decompressed).unwrap_or_default();
        maps.insert(tag, map);
    }
    maps
}

fn locale_maps() -> &'static HashMap<&'static str, HashMap<String, String>> {
    LOCALE_MAPS.get_or_init(load_locale_map)
}

pub use messages_generated::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum Category {
    #[default]
    Warning,
    Error,
    Suggestion,
    Message,
}

impl Category {
    pub fn name(self) -> &'static str {
        match self {
            Category::Warning => "warning",
            Category::Error => "error",
            Category::Suggestion => "suggestion",
            Category::Message => "message",
        }
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

pub type Key = &'static str;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Message {
    pub code: i32,
    pub category: Category,
    pub key: Key,
    pub text: &'static str,
    pub reports_unnecessary: bool,
    pub elided_in_compatibility_pyramid: bool,
    pub reports_deprecated: bool,
}

impl Message {
    pub fn code(&self) -> i32 {
        self.code
    }

    pub fn category(&self) -> Category {
        self.category
    }

    pub fn key(&self) -> Key {
        self.key
    }

    pub fn reports_unnecessary(&self) -> bool {
        self.reports_unnecessary
    }

    pub fn reports_deprecated(&self) -> bool {
        self.reports_deprecated
    }

    pub fn format(&self, args: &[&str]) -> String {
        format_message(self.text, args)
    }

    pub fn localize(&self, locale: &Locale, args: &[&str]) -> String {
        let locale_str = locale.as_str();
        let maps = locale_maps();
        let text = maps
            .get(locale_str)
            .and_then(|m| m.get(self.key))
            .or_else(|| {
                if let Some(lang) = locale_str.split('-').next() {
                    maps.get(lang).and_then(|m| m.get(self.key))
                } else {
                    None
                }
            })
            .map(String::as_str)
            .unwrap_or(self.text);
        format_message(text, args)
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.text)
    }
}

pub fn new_ad_hoc_message(text: &'static str) -> Message {
    Message {
        code: -1,
        category: Category::Error,
        key: "-1",
        text,
        reports_unnecessary: false,
        elided_in_compatibility_pyramid: false,
        reports_deprecated: false,
    }
}

pub fn format_message(text: &str, args: &[&str]) -> String {
    if args.is_empty() {
        return text.to_string();
    }

    let re = regex::Regex::new(r"\{(\d+)\}").expect("valid regex");
    re.replace_all(text, |caps: &regex::Captures| {
        let index: usize = caps
            .get(1)
            .expect("capture group 1")
            .as_str()
            .parse()
            .expect("Invalid formatting placeholder");
        if index >= args.len() {
            panic!("Invalid formatting placeholder");
        }
        args[index]
    })
    .into_owned()
}

#[cfg(test)]
mod tests;
