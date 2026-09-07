#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn ensure_host_globals(&mut self) {
        const DOM_VALUES: &[&str] = &[
            "document",
            "window",
            "navigator",
            "self",
            "top",
            "parent",
            "frames",
            "location",
            "history",
            "screen",
            "localStorage",
            "sessionStorage",
            "console",
            "alert",
            "confirm",
            "prompt",
            "fetch",
            "setTimeout",
            "setInterval",
            "clearTimeout",
            "clearInterval",
            "queueMicrotask",
            "requestAnimationFrame",
            "cancelAnimationFrame",
            "getComputedStyle",
            "matchMedia",
            "addEventListener",
            "removeEventListener",
            "postMessage",
            "atob",
            "btoa",
            "scrollTo",
            "scrollBy",
        ];

        const DOM_TYPES: &[&str] = &[
            "HTMLElement",
            "Element",
            "Node",
            "Event",
            "EventTarget",
            "Document",
            "DocumentFragment",
            "ShadowRoot",
            "Window",
            "NodeList",
            "HTMLInputElement",
            "HTMLButtonElement",
            "HTMLDivElement",
            "HTMLSpanElement",
            "HTMLAnchorElement",
            "HTMLFormElement",
            "HTMLSelectElement",
            "HTMLTextAreaElement",
            "HTMLCanvasElement",
            "CanvasRenderingContext2D",
            "MouseEvent",
            "KeyboardEvent",
            "DataTransfer",
            "SVGElement",
            "TrustedHTML",
            "StyleMedia",
            "FormData",
            "Blob",
            "File",
            "URL",
            "URLSearchParams",
            "TextEncoder",
            "TextDecoder",
            "AbortController",
            "AbortSignal",
            "Headers",
            "Request",
            "Response",
            "ReadableStream",
            "WritableStream",
            "TransformStream",
        ];

        const ES_TYPES: &[&str] = &[
            "Promise",
            "Iterable",
            "Iterator",
            "IterableIterator",
            "Symbol",
            "Generator",
            "AsyncIterable",
            "AsyncIterator",
            "Awaited",
            "ArrayBuffer",
            "Uint8Array",
            "Int8Array",
            "Uint16Array",
            "Int16Array",
            "Uint32Array",
            "Int32Array",
            "Float32Array",
            "Float64Array",
            "DataView",
            "Date",
            "Math",
            "Error",
            "Intl",
            "JSON",
            "Map",
            "Set",
            "WeakMap",
            "WeakSet",
            "TemplateStringsArray",
            "TypedPropertyDescriptor",
            "ReadonlyArray",
            "BigInt",
            "Proxy",
            "Reflect",
            "FinalizationRegistry",
            "WeakRef",
            "SharedArrayBuffer",
            "Atomics",
            "globalThis",
        ];

        const UTILITY_TYPES: &[&str] = &[
            "Partial",
            "Readonly",
            "Pick",
            "Record",
            "Omit",
            "Exclude",
            "Extract",
            "NonNullable",
            "Parameters",
            "ReturnType",
            "ConstructorParameters",
            "InstanceType",
            "Required",
            "ReadonlyArray",
        ];

        for &name in DOM_VALUES
            .iter()
            .chain(DOM_TYPES.iter())
            .chain(ES_TYPES.iter())
            .chain(UTILITY_TYPES.iter())
        {
            if self.globals.get(name).is_none() {
                self.globals.insert(
                    name.to_string(),
                    Arc::new(Symbol::new(SymbolFlags::Property, name)),
                );
            }
        }
    }

    pub(crate) fn ensure_jsx_namespace(&mut self) {
        use crate::checker::jsx::JsxNames;
        if !self.is_jsx_enabled() || self.get_jsx_namespace().is_some() {
            return;
        }

        let mut jsx = Symbol::new(SymbolFlags::NamespaceModule, JsxNames::JSX);

        let element = Symbol::new(SymbolFlags::TypeLiteral, JsxNames::ELEMENT);
        jsx.members
            .insert(JsxNames::ELEMENT.to_string(), Arc::new(element));

        let mut intrinsic = Symbol::new(SymbolFlags::TypeLiteral, JsxNames::INTRINSIC_ELEMENTS);
        intrinsic.members.insert(
            crate::ast::INTERNAL_SYMBOL_NAME_INDEX.to_string(),
            Arc::new(Symbol::new(SymbolFlags::TypeLiteral, "")),
        );
        jsx.members.insert(
            JsxNames::INTRINSIC_ELEMENTS.to_string(),
            Arc::new(intrinsic),
        );

        self.globals
            .insert(JsxNames::JSX.to_string(), Arc::new(jsx));
    }

    pub fn any_type(&self) -> Arc<Type> {
        self.any_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::Any,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "any".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn unknown_type(&self) -> Arc<Type> {
        self.unknown_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::Unknown,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "unknown".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn undefined_type(&self) -> Arc<Type> {
        self.undefined_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::Undefined,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "undefined".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub(crate) fn nullish_widening_type(&self, base: Arc<Type>) -> Arc<Type> {
        if self.strict_null_checks {
            return base;
        }
        let mut t = Type::new(
            base.flags,
            TypeData::Intrinsic(IntrinsicTypeData {
                intrinsic_name: base.intrinsic_name().unwrap_or("undefined").to_string(),
            }),
        );
        t.object_flags |= crate::checker::types::ObjectFlags::ContainsWideningType;
        Arc::new(t)
    }

    pub fn null_type(&self) -> Arc<Type> {
        self.null_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::Null,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "null".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn string_type(&self) -> Arc<Type> {
        self.string_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::String,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "string".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn number_type(&self) -> Arc<Type> {
        self.number_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::Number,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "number".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn bigint_type(&self) -> Arc<Type> {
        self.bigint_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::BigInt,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "bigint".to_string(),
                    }),
                ))
            })
            .clone()
    }
}
