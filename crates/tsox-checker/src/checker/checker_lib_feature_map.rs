// Go checker/utilities.go getFeatureMap：lib 特性属性表，
// 属性不存在（TS2550）与名字不存在（TS2583）时给出目标 lib 建议

struct FeatureMapEntry {
    lib: &'static str,
    props: &'static [&'static str],
}

struct ContainerFeatures {
    container: &'static str,
    entries: &'static [FeatureMapEntry],
}

const ARRAY_LIKE_ES2023: &[&str] = &[
    "findLastIndex",
    "findLast",
    "toReversed",
    "toSorted",
    "toSpliced",
    "with",
];

const TYPED_ARRAY_FEATURES: &[ContainerFeatures] = &[
    ContainerFeatures {
        container: "Int8Array",
        entries: &[
            FeatureMapEntry { lib: "es2022", props: &["at"] },
            FeatureMapEntry { lib: "es2023", props: ARRAY_LIKE_ES2023 },
        ],
    },
    ContainerFeatures {
        container: "Uint8Array",
        entries: &[
            FeatureMapEntry { lib: "es2022", props: &["at"] },
            FeatureMapEntry { lib: "es2023", props: ARRAY_LIKE_ES2023 },
        ],
    },
    ContainerFeatures {
        container: "Uint8ClampedArray",
        entries: &[
            FeatureMapEntry { lib: "es2022", props: &["at"] },
            FeatureMapEntry { lib: "es2023", props: ARRAY_LIKE_ES2023 },
        ],
    },
    ContainerFeatures {
        container: "Int16Array",
        entries: &[
            FeatureMapEntry { lib: "es2022", props: &["at"] },
            FeatureMapEntry { lib: "es2023", props: ARRAY_LIKE_ES2023 },
        ],
    },
    ContainerFeatures {
        container: "Uint16Array",
        entries: &[
            FeatureMapEntry { lib: "es2022", props: &["at"] },
            FeatureMapEntry { lib: "es2023", props: ARRAY_LIKE_ES2023 },
        ],
    },
    ContainerFeatures {
        container: "Int32Array",
        entries: &[
            FeatureMapEntry { lib: "es2022", props: &["at"] },
            FeatureMapEntry { lib: "es2023", props: ARRAY_LIKE_ES2023 },
        ],
    },
    ContainerFeatures {
        container: "Uint32Array",
        entries: &[
            FeatureMapEntry { lib: "es2022", props: &["at"] },
            FeatureMapEntry { lib: "es2023", props: ARRAY_LIKE_ES2023 },
        ],
    },
    ContainerFeatures {
        container: "Float32Array",
        entries: &[
            FeatureMapEntry { lib: "es2022", props: &["at"] },
            FeatureMapEntry { lib: "es2023", props: ARRAY_LIKE_ES2023 },
        ],
    },
    ContainerFeatures {
        container: "Float64Array",
        entries: &[
            FeatureMapEntry { lib: "es2022", props: &["at"] },
            FeatureMapEntry { lib: "es2023", props: ARRAY_LIKE_ES2023 },
        ],
    },
    ContainerFeatures {
        container: "BigInt64Array",
        entries: &[
            FeatureMapEntry { lib: "es2020", props: &[] },
            FeatureMapEntry { lib: "es2022", props: &["at"] },
            FeatureMapEntry { lib: "es2023", props: ARRAY_LIKE_ES2023 },
        ],
    },
    ContainerFeatures {
        container: "BigUint64Array",
        entries: &[
            FeatureMapEntry { lib: "es2020", props: &[] },
            FeatureMapEntry { lib: "es2022", props: &["at"] },
            FeatureMapEntry { lib: "es2023", props: ARRAY_LIKE_ES2023 },
        ],
    },
];

const FEATURE_MAP: &[ContainerFeatures] = &[
    ContainerFeatures {
        container: "Array",
        entries: &[
            FeatureMapEntry {
                lib: "es2015",
                props: &["find", "findIndex", "fill", "copyWithin", "entries", "keys", "values"],
            },
            FeatureMapEntry { lib: "es2016", props: &["includes"] },
            FeatureMapEntry { lib: "es2019", props: &["flat", "flatMap"] },
            FeatureMapEntry { lib: "es2022", props: &["at"] },
            FeatureMapEntry { lib: "es2023", props: ARRAY_LIKE_ES2023 },
        ],
    },
    ContainerFeatures {
        container: "Iterator",
        entries: &[FeatureMapEntry { lib: "es2015", props: &[] }],
    },
    ContainerFeatures {
        container: "AsyncIterator",
        entries: &[FeatureMapEntry { lib: "es2015", props: &[] }],
    },
    ContainerFeatures {
        container: "ArrayBuffer",
        entries: &[FeatureMapEntry {
            lib: "es2024",
            props: &[
                "maxByteLength",
                "resizable",
                "resize",
                "detached",
                "transfer",
                "transferToFixedLength",
            ],
        }],
    },
    ContainerFeatures {
        container: "Atomics",
        entries: &[
            FeatureMapEntry {
                lib: "es2017",
                props: &[
                    "add",
                    "and",
                    "compareExchange",
                    "exchange",
                    "isLockFree",
                    "load",
                    "or",
                    "store",
                    "sub",
                    "wait",
                    "notify",
                    "xor",
                ],
            },
            FeatureMapEntry { lib: "es2024", props: &["waitAsync"] },
        ],
    },
    ContainerFeatures {
        container: "SharedArrayBuffer",
        entries: &[
            FeatureMapEntry { lib: "es2017", props: &["byteLength", "slice"] },
            FeatureMapEntry {
                lib: "es2024",
                props: &["growable", "maxByteLength", "grow"],
            },
        ],
    },
    ContainerFeatures {
        container: "AsyncIterable",
        entries: &[FeatureMapEntry { lib: "es2018", props: &[] }],
    },
    ContainerFeatures {
        container: "AsyncIterableIterator",
        entries: &[FeatureMapEntry { lib: "es2018", props: &[] }],
    },
    ContainerFeatures {
        container: "AsyncGenerator",
        entries: &[FeatureMapEntry { lib: "es2018", props: &[] }],
    },
    ContainerFeatures {
        container: "AsyncGeneratorFunction",
        entries: &[FeatureMapEntry { lib: "es2018", props: &[] }],
    },
    ContainerFeatures {
        container: "RegExp",
        entries: &[
            FeatureMapEntry { lib: "es2015", props: &["flags", "sticky", "unicode"] },
            FeatureMapEntry { lib: "es2018", props: &["dotAll"] },
            FeatureMapEntry { lib: "es2024", props: &["unicodeSets"] },
        ],
    },
    ContainerFeatures {
        container: "RegExpConstructor",
        entries: &[FeatureMapEntry { lib: "es2025", props: &["escape"] }],
    },
    ContainerFeatures {
        container: "Reflect",
        entries: &[FeatureMapEntry {
            lib: "es2015",
            props: &[
                "apply",
                "construct",
                "defineProperty",
                "deleteProperty",
                "get",
                "getOwnPropertyDescriptor",
                "getPrototypeOf",
                "has",
                "isExtensible",
                "ownKeys",
                "preventExtensions",
                "set",
                "setPrototypeOf",
            ],
        }],
    },
    ContainerFeatures {
        container: "ArrayConstructor",
        entries: &[
            FeatureMapEntry { lib: "es2015", props: &["from", "of"] },
            FeatureMapEntry { lib: "esnext", props: &["fromAsync"] },
        ],
    },
    ContainerFeatures {
        container: "ObjectConstructor",
        entries: &[
            FeatureMapEntry {
                lib: "es2015",
                props: &["assign", "getOwnPropertySymbols", "keys", "is", "setPrototypeOf"],
            },
            FeatureMapEntry {
                lib: "es2017",
                props: &["values", "entries", "getOwnPropertyDescriptors"],
            },
            FeatureMapEntry { lib: "es2019", props: &["fromEntries"] },
            FeatureMapEntry { lib: "es2022", props: &["hasOwn"] },
            FeatureMapEntry { lib: "es2024", props: &["groupBy"] },
        ],
    },
    ContainerFeatures {
        container: "NumberConstructor",
        entries: &[FeatureMapEntry {
            lib: "es2015",
            props: &[
                "isFinite",
                "isInteger",
                "isNaN",
                "isSafeInteger",
                "parseFloat",
                "parseInt",
            ],
        }],
    },
    ContainerFeatures {
        container: "Math",
        entries: &[
            FeatureMapEntry {
                lib: "es2015",
                props: &[
                    "clz32",
                    "imul",
                    "sign",
                    "log10",
                    "log2",
                    "log1p",
                    "expm1",
                    "cosh",
                    "sinh",
                    "tanh",
                    "acosh",
                    "asinh",
                    "atanh",
                    "hypot",
                    "trunc",
                    "fround",
                    "cbrt",
                ],
            },
            FeatureMapEntry { lib: "es2025", props: &["f16round"] },
        ],
    },
    ContainerFeatures {
        container: "Map",
        entries: &[
            FeatureMapEntry { lib: "es2015", props: &["entries", "keys", "values"] },
            FeatureMapEntry {
                lib: "esnext",
                props: &["getOrInsert", "getOrInsertComputed"],
            },
        ],
    },
    ContainerFeatures {
        container: "MapConstructor",
        entries: &[FeatureMapEntry { lib: "es2024", props: &["groupBy"] }],
    },
    ContainerFeatures {
        container: "Set",
        entries: &[
            FeatureMapEntry { lib: "es2015", props: &["entries", "keys", "values"] },
            FeatureMapEntry {
                lib: "es2025",
                props: &[
                    "union",
                    "intersection",
                    "difference",
                    "symmetricDifference",
                    "isSubsetOf",
                    "isSupersetOf",
                    "isDisjointFrom",
                ],
            },
        ],
    },
    ContainerFeatures {
        container: "PromiseConstructor",
        entries: &[
            FeatureMapEntry {
                lib: "es2015",
                props: &["all", "race", "reject", "resolve"],
            },
            FeatureMapEntry { lib: "es2020", props: &["allSettled"] },
            FeatureMapEntry { lib: "es2021", props: &["any"] },
            FeatureMapEntry { lib: "es2024", props: &["withResolvers"] },
            FeatureMapEntry { lib: "es2025", props: &["try"] },
        ],
    },
    ContainerFeatures {
        container: "Symbol",
        entries: &[
            FeatureMapEntry { lib: "es2015", props: &["for", "keyFor"] },
            FeatureMapEntry { lib: "es2019", props: &["description"] },
        ],
    },
    ContainerFeatures {
        container: "WeakMap",
        entries: &[
            FeatureMapEntry { lib: "es2015", props: &[] },
            FeatureMapEntry {
                lib: "esnext",
                props: &["getOrInsert", "getOrInsertComputed"],
            },
        ],
    },
    ContainerFeatures {
        container: "WeakSet",
        entries: &[FeatureMapEntry { lib: "es2015", props: &[] }],
    },
    ContainerFeatures {
        container: "String",
        entries: &[
            FeatureMapEntry {
                lib: "es2015",
                props: &[
                    "codePointAt",
                    "includes",
                    "endsWith",
                    "normalize",
                    "repeat",
                    "startsWith",
                    "anchor",
                    "big",
                    "blink",
                    "bold",
                    "fixed",
                    "fontcolor",
                    "fontsize",
                    "italics",
                    "link",
                    "small",
                    "strike",
                    "sub",
                    "sup",
                ],
            },
            FeatureMapEntry { lib: "es2017", props: &["padStart", "padEnd"] },
            FeatureMapEntry {
                lib: "es2019",
                props: &["trimStart", "trimEnd", "trimLeft", "trimRight"],
            },
            FeatureMapEntry { lib: "es2020", props: &["matchAll"] },
            FeatureMapEntry { lib: "es2021", props: &["replaceAll"] },
            FeatureMapEntry { lib: "es2022", props: &["at"] },
            FeatureMapEntry {
                lib: "es2024",
                props: &["isWellFormed", "toWellFormed"],
            },
        ],
    },
    ContainerFeatures {
        container: "StringConstructor",
        entries: &[FeatureMapEntry {
            lib: "es2015",
            props: &["fromCodePoint", "raw"],
        }],
    },
    ContainerFeatures {
        container: "DateTimeFormat",
        entries: &[FeatureMapEntry {
            lib: "es2017",
            props: &["formatToParts"],
        }],
    },
    ContainerFeatures {
        container: "Promise",
        entries: &[
            FeatureMapEntry { lib: "es2015", props: &[] },
            FeatureMapEntry { lib: "es2018", props: &["finally"] },
        ],
    },
    ContainerFeatures {
        container: "RegExpMatchArray",
        entries: &[FeatureMapEntry { lib: "es2018", props: &["groups"] }],
    },
    ContainerFeatures {
        container: "RegExpExecArray",
        entries: &[FeatureMapEntry { lib: "es2018", props: &["groups"] }],
    },
    ContainerFeatures {
        container: "Intl",
        entries: &[
            FeatureMapEntry { lib: "es2018", props: &["PluralRules"] },
            FeatureMapEntry {
                lib: "es2020",
                props: &["RelativeTimeFormat", "Locale", "DisplayNames"],
            },
            FeatureMapEntry {
                lib: "es2021",
                props: &["ListFormat", "DateTimeFormat"],
            },
            FeatureMapEntry { lib: "es2022", props: &["Segmenter"] },
            FeatureMapEntry { lib: "es2025", props: &["DurationFormat"] },
        ],
    },
    ContainerFeatures {
        container: "NumberFormat",
        entries: &[FeatureMapEntry {
            lib: "es2018",
            props: &["formatToParts"],
        }],
    },
    ContainerFeatures {
        container: "SymbolConstructor",
        entries: &[
            FeatureMapEntry { lib: "es2020", props: &["matchAll"] },
            FeatureMapEntry {
                lib: "esnext",
                props: &["metadata", "dispose", "asyncDispose"],
            },
        ],
    },
    ContainerFeatures {
        container: "DataView",
        entries: &[
            FeatureMapEntry {
                lib: "es2020",
                props: &["setBigInt64", "setBigUint64", "getBigInt64", "getBigUint64"],
            },
            FeatureMapEntry { lib: "es2025", props: &["setFloat16", "getFloat16"] },
        ],
    },
    ContainerFeatures {
        container: "BigInt",
        entries: &[FeatureMapEntry { lib: "es2020", props: &[] }],
    },
    ContainerFeatures {
        container: "RelativeTimeFormat",
        entries: &[FeatureMapEntry {
            lib: "es2020",
            props: &["format", "formatToParts", "resolvedOptions"],
        }],
    },
    ContainerFeatures {
        container: "Float16Array",
        entries: &[FeatureMapEntry { lib: "es2025", props: &[] }],
    },
    ContainerFeatures {
        container: "Error",
        entries: &[FeatureMapEntry { lib: "es2022", props: &["cause"] }],
    },
    ContainerFeatures {
        container: "ErrorConstructor",
        entries: &[FeatureMapEntry { lib: "esnext", props: &["isError"] }],
    },
    ContainerFeatures {
        container: "Uint8ArrayConstructor",
        entries: &[FeatureMapEntry {
            lib: "esnext",
            props: &["fromBase64", "fromHex"],
        }],
    },
    ContainerFeatures {
        container: "DisposableStack",
        entries: &[FeatureMapEntry { lib: "esnext", props: &[] }],
    },
];

fn container_features(container: &str) -> Option<&'static [FeatureMapEntry]> {
    FEATURE_MAP
        .iter()
        .chain(TYPED_ARRAY_FEATURES.iter())
        .find(|c| c.container == container)
        .map(|c| c.entries)
}

// Go getSuggestedLibForNonExistentProperty：属性名命中容器特性表时给出
// 引入该属性的最低 lib
pub(crate) fn suggested_lib_for_property(container: &str, prop: &str) -> Option<&'static str> {
    container_features(container)?
        .iter()
        .find(|e| e.props.contains(&prop))
        .map(|e| e.lib)
}

// Go getSuggestedLibForNonExistentName：名字本身是 lib 提供的全局时给出
// 首个提供该名字的 lib
pub(crate) fn suggested_lib_for_name(name: &str) -> Option<&'static str> {
    let entries = container_features(name)?;
    entries.first().map(|e| e.lib)
}

use crate::checker::checker::Checker;
use crate::checker::types::*;
use std::sync::Arc;

impl Checker {
    // Go getApparentType 的符号名语义：primitive 及字面量映射到全局包装接口名，
    // 其余取类型符号名（用于 lib 特性表查找）
    pub(crate) fn lib_suggestion_container_name(&self, t: &Arc<Type>) -> Option<String> {
        if let Some(sym) = t.symbol.as_ref() {
            return Some(sym.name.clone());
        }
        let name = if t.flags.contains(TypeFlags::String) || t.flags.contains(TypeFlags::StringLiteral) {
            "String"
        } else if t.flags.contains(TypeFlags::Number) || t.flags.contains(TypeFlags::NumberLiteral) {
            "Number"
        } else if t.flags.contains(TypeFlags::Boolean) || t.flags.contains(TypeFlags::BooleanLiteral) {
            "Boolean"
        } else if t.flags.contains(TypeFlags::BigInt) || t.flags.contains(TypeFlags::BigIntLiteral) {
            "BigInt"
        } else if t.flags.contains(TypeFlags::ESSymbol) || t.flags.contains(TypeFlags::UniqueESSymbol) {
            "Symbol"
        } else {
            return None;
        };
        Some(name.to_string())
    }
}
