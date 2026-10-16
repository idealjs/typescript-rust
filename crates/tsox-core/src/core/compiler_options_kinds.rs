#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
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
    pub fn is_non_node_esm(&self) -> bool { crate::fntrace::enter("is_non_node_esm"); 
        *self >= ModuleKind::ES2015 && *self <= ModuleKind::ESNext
    }

    pub fn supports_import_attributes(&self) -> bool { crate::fntrace::enter("supports_import_attributes"); 
        (*self >= ModuleKind::Node18 && *self <= ModuleKind::NodeNext)
            || *self == ModuleKind::Preserve
            || *self == ModuleKind::ESNext
    }
}

impl std::fmt::Display for ModuleKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { crate::fntrace::enter("fmt"); 
        let name = match self {
            ModuleKind::None => "None",
            ModuleKind::CommonJS => "CommonJS",
            ModuleKind::AMD => "AMD",
            ModuleKind::UMD => "UMD",
            ModuleKind::System => "System",
            ModuleKind::ES2015 => "ES2015",
            ModuleKind::ES2020 => "ES2020",
            ModuleKind::ES2022 => "ES2022",
            ModuleKind::ESNext => "ESNext",
            ModuleKind::Node16 => "Node16",
            ModuleKind::Node18 => "Node18",
            ModuleKind::Node20 => "Node20",
            ModuleKind::NodeNext => "NodeNext",
            ModuleKind::Preserve => "Preserve",
        };
        f.write_str(name)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { crate::fntrace::enter("fmt"); 
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[repr(i32)]
pub enum ModuleDetectionKind {
    #[default]
    None = 0,
    Auto = 1,
    Legacy = 2,
    Force = 3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { crate::fntrace::enter("fmt"); 
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[repr(i32)]
pub enum NewLineKind {
    #[default]
    None = 0,
    CRLF = 1,
    LF = 2,
}

impl NewLineKind {
    pub fn from_str(s: &str) -> NewLineKind { crate::fntrace::enter("from_str"); 
        match s {
            "\r\n" => NewLineKind::CRLF,
            "\n" => NewLineKind::LF,
            _ => NewLineKind::None,
        }
    }

    pub fn get_new_line_character(&self) -> &'static str { crate::fntrace::enter("get_new_line_character"); 
        match self {
            NewLineKind::CRLF => "\r\n",
            _ => "\n",
        }
    }
}

impl From<i64> for ScriptTarget {
    fn from(v: i64) -> Self { crate::fntrace::enter("from"); 
        match v {
            1 => ScriptTarget::ES5,
            2 => ScriptTarget::ES2015,
            3 => ScriptTarget::ES2016,
            4 => ScriptTarget::ES2017,
            5 => ScriptTarget::ES2018,
            6 => ScriptTarget::ES2019,
            7 => ScriptTarget::ES2020,
            8 => ScriptTarget::ES2021,
            9 => ScriptTarget::ES2022,
            10 => ScriptTarget::ES2023,
            11 => ScriptTarget::ES2024,
            12 => ScriptTarget::ES2025,
            99 => ScriptTarget::ESNext,
            100 => ScriptTarget::JSON,
            _ => ScriptTarget::None,
        }
    }
}

impl From<i64> for ModuleKind {
    fn from(v: i64) -> Self { crate::fntrace::enter("from"); 
        match v {
            1 => ModuleKind::CommonJS,
            2 => ModuleKind::AMD,
            3 => ModuleKind::UMD,
            4 => ModuleKind::System,
            5 => ModuleKind::ES2015,
            6 => ModuleKind::ES2020,
            7 => ModuleKind::ES2022,
            99 => ModuleKind::ESNext,
            100 => ModuleKind::Node16,
            101 => ModuleKind::Node18,
            102 => ModuleKind::Node20,
            199 => ModuleKind::NodeNext,
            200 => ModuleKind::Preserve,
            _ => ModuleKind::None,
        }
    }
}

impl From<i64> for ModuleResolutionKind {
    fn from(v: i64) -> Self { crate::fntrace::enter("from"); 
        match v {
            1 => ModuleResolutionKind::Classic,
            2 => ModuleResolutionKind::Node10,
            3 => ModuleResolutionKind::Node16,
            99 => ModuleResolutionKind::NodeNext,
            100 => ModuleResolutionKind::Bundler,
            _ => ModuleResolutionKind::Unknown,
        }
    }
}

impl From<i64> for ModuleDetectionKind {
    fn from(v: i64) -> Self { crate::fntrace::enter("from"); 
        match v {
            1 => ModuleDetectionKind::Auto,
            2 => ModuleDetectionKind::Legacy,
            3 => ModuleDetectionKind::Force,
            _ => ModuleDetectionKind::None,
        }
    }
}

impl From<i64> for JsxEmit {
    fn from(v: i64) -> Self { crate::fntrace::enter("from"); 
        match v {
            1 => JsxEmit::Preserve,
            2 => JsxEmit::ReactNative,
            3 => JsxEmit::React,
            4 => JsxEmit::ReactJSX,
            5 => JsxEmit::ReactJSXDev,
            _ => JsxEmit::None,
        }
    }
}

impl From<i64> for NewLineKind {
    fn from(v: i64) -> Self { crate::fntrace::enter("from"); 
        match v {
            1 => NewLineKind::CRLF,
            2 => NewLineKind::LF,
            _ => NewLineKind::None,
        }
    }
}
