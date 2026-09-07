#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i32)]
pub enum ScriptTarget {
    #[default]
    None = 0,
    ES5 = 1,
    ES2015 = 2,
    ES2016 = 3,
    ES2017 = 4,
    ES2018 = 5,
    ES2019 = 6,
    ES2020 = 7,
    ES2021 = 8,
    ES2022 = 9,
    ES2023 = 10,
    ES2024 = 11,
    ES2025 = 12,
    ESNext = 99,
    JSON = 100,
}

impl ScriptTarget {
    pub const LATEST: ScriptTarget = ScriptTarget::ESNext;
    pub const LATEST_STANDARD: ScriptTarget = ScriptTarget::ES2025;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i32)]
pub enum ModuleKind {
    #[default]
    None = 0,
    CommonJS = 1,
    AMD = 2,
    UMD = 3,
    System = 4,
    ES2015 = 5,
    ES2020 = 6,
    ES2022 = 7,
    ESNext = 99,
    Node16 = 100,
    Node18 = 101,
    Node20 = 102,
    NodeNext = 199,
    Preserve = 200,
}

impl ModuleKind {
    pub fn is_non_node_esm(&self) -> bool {
        *self >= ModuleKind::ES2015 && *self <= ModuleKind::ESNext
    }

    pub fn supports_import_attributes(&self) -> bool {
        (*self >= ModuleKind::Node18 && *self <= ModuleKind::NodeNext)
            || *self == ModuleKind::Preserve
            || *self == ModuleKind::ESNext
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ModuleResolutionKind {
    #[default]
    Unknown = 0,
    Classic = 1,
    Node10 = 2,
    Node16 = 3,
    NodeNext = 99,
    Bundler = 100,
}

impl std::fmt::Display for ModuleResolutionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleResolutionKind::Unknown => write!(f, "Unknown"),
            ModuleResolutionKind::Classic => write!(f, "Classic"),
            ModuleResolutionKind::Node10 => write!(f, "Node10"),
            ModuleResolutionKind::Node16 => write!(f, "Node16"),
            ModuleResolutionKind::NodeNext => write!(f, "NodeNext"),
            ModuleResolutionKind::Bundler => write!(f, "Bundler"),
        }
    }
}

pub type ResolutionMode = ModuleKind;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ModuleDetectionKind {
    #[default]
    None = 0,
    Auto = 1,
    Legacy = 2,
    Force = 3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum JsxEmit {
    #[default]
    None = 0,
    Preserve = 1,
    ReactNative = 2,
    React = 3,
    ReactJSX = 4,
    ReactJSXDev = 5,
}

impl std::fmt::Display for JsxEmit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsxEmit::None => write!(f, "none"),
            JsxEmit::Preserve => write!(f, "preserve"),
            JsxEmit::ReactNative => write!(f, "react-native"),
            JsxEmit::React => write!(f, "react"),
            JsxEmit::ReactJSX => write!(f, "react-jsx"),
            JsxEmit::ReactJSXDev => write!(f, "react-jsxdev"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum NewLineKind {
    #[default]
    None = 0,
    CRLF = 1,
    LF = 2,
}

impl NewLineKind {
    pub fn from_str(s: &str) -> NewLineKind {
        match s {
            "\r\n" => NewLineKind::CRLF,
            "\n" => NewLineKind::LF,
            _ => NewLineKind::None,
        }
    }

    pub fn get_new_line_character(&self) -> &'static str {
        match self {
            NewLineKind::CRLF => "\r\n",
            _ => "\n",
        }
    }
}
