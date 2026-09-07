use super::*;
use crate::bundled::{BundledFS, lib_path};
use crate::compiler::{CompilerHost, CompilerHostImpl, Program, ProgramOptions};
use crate::core::compiler_options::CompilerOptions;
use crate::tsoptions::ParsedCommandLine;
use crate::vfs::InMemoryFS;

pub(crate) fn check_files(
    files: &[(&str, &str)],
    root: &str,
    configure: impl FnOnce(&mut CompilerOptions),
) -> Vec<i32> {
    let inner = Arc::new(InMemoryFS::new());
    inner.insert_dir("/proj");
    for (name, content) in files {
        let abs = if name.starts_with('/') {
            (*name).to_string()
        } else {
            format!("/proj/{name}")
        };

        let mut parent = crate::tspath::get_directory_path(&abs);
        loop {
            inner.insert_dir(&parent);
            let next = crate::tspath::get_directory_path(&parent);
            if next == parent {
                break;
            }
            parent = next;
        }
        inner.insert_file(&abs, content);
    }
    let fs = Arc::new(BundledFS::new(inner));
    let mut options = CompilerOptions::default();
    configure(&mut options);
    let parsed = ParsedCommandLine {
        file_names: vec![root.to_string()],
        compiler_options: options,
        ..Default::default()
    };
    let host: Arc<dyn CompilerHost> =
        Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()));
    let program = Arc::new(Program::new(ProgramOptions {
        config: parsed,
        host,
    }));
    let checker = program.build_checker();

    checker
        .diagnostics
        .get_all()
        .iter()
        .chain(program.diagnostics().iter().map(|d| d.as_ref()))
        .filter(|d| d.file.as_ref().is_some_and(|f| f.file_name == root))
        .map(|d| d.code)
        .collect()
}

#[test]
fn import_meta_reports_1470_only_in_cjs_files() {
    let files = [
        (
            "/proj/package.json",
            r#"{"name": "package", "type": "module"}"#,
        ),
        ("/proj/sub/package.json", r#"{"type": "commonjs"}"#),
        (
            "/proj/sub/index.ts",
            "const x = import.meta.url;\nexport {x};\n",
        ),
        (
            "/proj/index.ts",
            "const x = import.meta.url;\nexport {x};\n",
        ),
    ];
    let cjs = check_files(&files, "/proj/sub/index.ts", |o| {
        o.module = ModuleKind::Node16;
        o.module_resolution = ModuleResolutionKind::Node16;
    });
    assert_eq!(cjs, vec![1470], "CJS-format file must report TS1470");
    let esm = check_files(&files, "/proj/index.ts", |o| {
        o.module = ModuleKind::Node16;
        o.module_resolution = ModuleResolutionKind::Node16;
    });
    assert_eq!(esm, Vec::<i32>::new(), "ESM-format file must be clean");
}

#[test]
fn declare_global_augmentation_from_types_reference_merges() {
    let files = [
        (
            "/node_modules/pkg/package.json",
            r#"{ "name": "pkg", "exports": { "import": "./import.js", "require": "./require.js" } }"#,
        ),
        (
            "/node_modules/pkg/import.d.ts",
            "export {};\ndeclare global { var foo: number; }\n",
        ),
        (
            "/node_modules/pkg/require.d.ts",
            "export {};\ndeclare global { var bar: number; }\n",
        ),
        ("/package.json", r#"{ "type": "module" }"#),
        (
            "/index.ts",
            "/// <reference types=\"pkg\" resolution-mode=\"import\" />\nfoo;\nexport {};\n",
        ),
    ];

    let codes = check_files(&files, "/index.ts", |o| {
        o.module = ModuleKind::Node16;
        o.module_resolution = ModuleResolutionKind::Node16;
    });
    assert_eq!(
        codes,
        Vec::<i32>::new(),
        "foo must resolve via the global augmentation"
    );

    let files_both = [
        (
            "/node_modules/pkg/package.json",
            r#"{ "name": "pkg", "exports": { "import": "./import.js", "require": "./require.js" } }"#,
        ),
        (
            "/node_modules/pkg/import.d.ts",
            "export {};\ndeclare global { var foo: number; }\n",
        ),
        (
            "/node_modules/pkg/require.d.ts",
            "export {};\ndeclare global { var bar: number; }\n",
        ),
        ("/package.json", r#"{ "type": "module" }"#),
        (
            "/index.ts",
            "/// <reference types=\"pkg\" resolution-mode=\"import\" />\n/// <reference types=\"pkg\" resolution-mode=\"require\" />\nfoo;\nbar;\nexport {};\n",
        ),
    ];
    let codes = check_files(&files_both, "/index.ts", |o| {
        o.module = ModuleKind::Node16;
        o.module_resolution = ModuleResolutionKind::Node16;
    });
    assert_eq!(codes, Vec::<i32>::new(), "both augmentations must merge");

    let files_none = [
        (
            "/node_modules/pkg/package.json",
            r#"{ "name": "pkg", "exports": { "import": "./import.js" } }"#,
        ),
        (
            "/node_modules/pkg/import.d.ts",
            "export {};\ndeclare global { var foo: number; }\n",
        ),
        ("/package.json", r#"{ "type": "module" }"#),
        ("/index.ts", "foo;\nexport {};\n"),
    ];
    let codes = check_files(&files_none, "/index.ts", |o| {
        o.module = ModuleKind::Node16;
        o.module_resolution = ModuleResolutionKind::Node16;
    });
    assert_eq!(codes, vec![2304], "unreferenced augmentation must not leak");
}

#[test]
fn import_helpers_missing_helper_name_reports_2343() {
    let files = [
        (
            "/types.d.ts",
            "declare module \"fs\";\ndeclare module \"tslib\" { export {}; }\n",
        ),
        ("/sub/package.json", r#"{ "type": "commonjs" }"#),
        (
            "/sub/index.ts",
            "/// <reference path=\"/types.d.ts\" />\nexport { default } from \"fs\";\n",
        ),
    ];
    let codes = check_files(&files, "/sub/index.ts", |o| {
        o.module = ModuleKind::Node16;
        o.module_resolution = ModuleResolutionKind::Node16;
        o.import_helpers = crate::core::tristate::Tristate::True;
    });
    assert_eq!(
        codes,
        vec![2343],
        "missing __importDefault must report TS2343"
    );

    let files_ok = [
        (
            "/types.d.ts",
            "declare module \"fs\";\ndeclare module \"tslib\" { export function __importDefault(m: any): any; }\n",
        ),
        ("/sub/package.json", r#"{ "type": "commonjs" }"#),
        (
            "/sub/index.ts",
            "/// <reference path=\"/types.d.ts\" />\nexport { default } from \"fs\";\n",
        ),
    ];
    let codes = check_files(&files_ok, "/sub/index.ts", |o| {
        o.module = ModuleKind::Node16;
        o.module_resolution = ModuleResolutionKind::Node16;
        o.import_helpers = crate::core::tristate::Tristate::True;
    });
    assert_eq!(codes, Vec::<i32>::new(), "present helper must be clean");

    let files_esm = [
        (
            "/types.d.ts",
            "declare module \"fs\";\ndeclare module \"tslib\" { export {}; }\n",
        ),
        ("/package.json", r#"{ "type": "module" }"#),
        (
            "/index.ts",
            "/// <reference path=\"/types.d.ts\" />\nexport { default } from \"fs\";\n",
        ),
    ];
    let codes = check_files(&files_esm, "/index.ts", |o| {
        o.module = ModuleKind::Node16;
        o.module_resolution = ModuleResolutionKind::Node16;
        o.import_helpers = crate::core::tristate::Tristate::True;
    });
    assert_eq!(
        codes,
        Vec::<i32>::new(),
        "ESM-format emit needs no import helper"
    );

    let files_nointerop = [
        (
            "/types.d.ts",
            "declare module \"fs\";\ndeclare module \"tslib\" { export {}; }\n",
        ),
        ("/sub/package.json", r#"{ "type": "commonjs" }"#),
        (
            "/sub/index.ts",
            "/// <reference path=\"/types.d.ts\" />\nexport { default } from \"fs\";\n",
        ),
    ];
    let codes = check_files(&files_nointerop, "/sub/index.ts", |o| {
        o.module = ModuleKind::Node16;
        o.module_resolution = ModuleResolutionKind::Node16;
        o.import_helpers = crate::core::tristate::Tristate::True;
        o.es_module_interop = crate::core::tristate::Tristate::False;
    });
    assert_eq!(
        codes,
        Vec::<i32>::new(),
        "explicit interop=false must disable the check"
    );
}

#[test]
fn module_member_check_2305_and_2459() {
    let files = [
        (
            "/mod.ts",
            "export interface A {}\n\
                 export const v = 1;\n\
                 interface Internal {}\n\
                 const notExportedConst = 2;\n\
                 export type T = number;\n",
        ),
        (
            "/main.ts",
            "import { A, Missing, Internal, notExportedConst, Missing2 } from \"./mod\";\n\
                 export { Nope } from \"./mod\";\n",
        ),
    ];
    let codes = check_files(&files, "/main.ts", |_| {});

    assert_eq!(
        codes,
        vec![2305, 2459, 2459, 2305, 2305],
        "missing members report 2305 (incl. re-export), module-locals 2459"
    );
}

#[test]
fn shorthand_ambient_module_members_exempt_from_2305() {
    let files = [
        (
            "/types.d.ts",
            "declare module \"short\";\ndeclare module \"real\" { export const v = 1; }\n",
        ),
        (
            "/main.ts",
            "/// <reference path=\"/types.d.ts\" />\n\
                 import { anything } from \"short\";\n\
                 export { whatever } from \"short\";\n\
                 import { missing } from \"real\";\n",
        ),
    ];
    let codes = check_files(&files, "/main.ts", |_| {});
    assert_eq!(
        codes,
        vec![2305],
        "shorthand ambient members resolve silently; non-shorthand ambient still checks"
    );
}

#[test]
fn module_member_check_default_export_forms() {
    let files = [
        ("/a1.ts", "export default class A {}\n"),
        ("/a2.ts", "export default class {}\n"),
        ("/a3.ts", "export default function f() {}\n"),
        ("/a4.ts", "export default function () {}\n"),
        (
            "/main.ts",
            "import { default as D1 } from \"./a1\";\n\
                 import { default as D2 } from \"./a2\";\n\
                 import { default as D3 } from \"./a3\";\n\
                 import { default as D4 } from \"./a4\";\n\
                 void [D1, D2, D3, D4];\n",
        ),
    ];
    let codes = check_files(&files, "/main.ts", |_| {});
    assert_eq!(
        codes,
        Vec::<i32>::new(),
        "named and anonymous default declarations answer a 'default' member import"
    );
}

#[test]
fn module_member_check_export_clauses() {
    let files = [
        ("/lib.ts", "export const X = 1;\n"),
        ("/local.ts", "const Y = 2;\nexport { Y };\n"),
        ("/fwd.ts", "export { X } from \"./lib\";\n"),
        ("/def.ts", "export { X as default } from \"./lib\";\n"),
        (
            "/main.ts",
            "import { Y } from \"./local\";\n\
                 import { X } from \"./fwd\";\n\
                 import { default as D } from \"./def\";\n\
                 import { Nope } from \"./local\";\n\
                 void [Y, X, D];\n",
        ),
    ];
    let codes = check_files(&files, "/main.ts", |_| {});
    assert_eq!(
        codes,
        vec![2305],
        "clause exports resolve in all three forms; unknown names still report 2305"
    );
}

#[test]
fn module_member_check_star_chains() {
    let files = [
        (
            "/leaf.ts",
            "export const deep = 1;\nexport const other = 2;\n",
        ),
        ("/mid.ts", "export * from \"./leaf\";\n"),
        (
            "/cyc1.ts",
            "export * from \"./cyc2\";\nexport const c1 = 1;\n",
        ),
        (
            "/cyc2.ts",
            "export * from \"./cyc1\";\nexport const c2 = 2;\n",
        ),
        (
            "/shadow.ts",
            "export * from \"./leaf\";\nexport const other = \"own\";\n",
        ),
        ("/star.ts", "export * from \"./leaf\";\n"),
        (
            "/main.ts",
            "import { deep } from \"./mid\";\n\
                 import { c1 } from \"./cyc2\";\n\
                 import { c2 } from \"./cyc1\";\n\
                 import { other } from \"./shadow\";\n\
                 import { default as D } from \"./star\";\n\
                 void [deep, c1, c2, other, D];\n",
        ),
    ];
    let codes = check_files(&files, "/main.ts", |_| {});
    assert_eq!(
        codes,
        vec![2305],
        "star chains resolve transitively and through cycles; 'default' never passes a star"
    );
}

#[test]
fn module_member_check_ambient_implicit_exports() {
    let files = [
        (
            "/types.d.ts",
            "declare module \"amb\" {\n    function f(): void;\n    interface I { x: number }\n    const v: number;\n}\n\
                 declare module \"exp\" {\n    export const e = 1;\n    function hidden(): void;\n}\n",
        ),
        (
            "/main.ts",
            "/// <reference path=\"/types.d.ts\" />\n\
                 import { f, I, v } from \"amb\";\n\
                 import { e, hidden } from \"exp\";\n\
                 void [f, v, e, hidden];\n",
        ),
    ];
    let codes = check_files(&files, "/main.ts", |_| {});
    assert_eq!(
        codes,
        Vec::<i32>::new(),
        "ambient module bodies implicitly export all declarations (export-const members don't break the context)"
    );
}

#[test]
fn module_member_check_export_equals_targets() {
    let files = [
        (
            "/thing.d.ts",
            "declare namespace Foo {\n    export interface Bar {}\n    export function f(): Bar;\n}\nexport = Foo;\n",
        ),
        (
            "/demo.d.ts",
            "declare namespace demoNS {\n    function g(): void;\n}\n\
                 declare module 'demoModule' {\n    import alias = demoNS;\n    export = alias;\n}\n",
        ),
        (
            "/main.ts",
            "/// <reference path=\"/thing.d.ts\" />\n\
                 /// <reference path=\"/demo.d.ts\" />\n\
                 import { f } from \"./thing\";\n\
                 import { g } from \"demoModule\";\n\
                 void [f, g];\n",
        ),
    ];
    let codes = check_files(&files, "/main.ts", |_| {});
    assert_eq!(
        codes,
        Vec::<i32>::new(),
        "export= namespace members resolve (direct and via import-alias), ambient namespace locals included"
    );
}

#[test]
fn module_member_check_synthetic_default() {
    let files = [
        (
            "/nodefault.d.ts",
            "export declare function helper(): void;\n",
        ),
        ("/plain.ts", "export const x = 1;\n"),
        (
            "/main.ts",
            "import { default as D1 } from \"./nodefault\";\n\
                 import { default as D2 } from \"./plain\";\n\
                 void [D1, D2];\n",
        ),
    ];
    let codes = check_files(&files, "/main.ts", |_| {});
    assert_eq!(
        codes,
        vec![2305],
        "declaration files answer a synthetic default; plain .ts modules without export= do not"
    );
}

#[test]
fn module_member_check_non_type_only_ignores_resolution_mode() {
    let files = [
        (
            "/node_modules/pkg/package.json",
            r#"{ "name": "pkg", "exports": { "import": "./import.js", "require": "./require.js" } }"#,
        ),
        (
            "/node_modules/pkg/import.d.ts",
            "export interface ImportInterface {}\n",
        ),
        (
            "/node_modules/pkg/require.d.ts",
            "export interface RequireInterface {}\n",
        ),
        (
            "/index.ts",
            "import type { ImportInterface } from \"pkg\" with { \"resolution-mode\": \"import\" };\n\
                 import { ImportInterface as Imp } from \"pkg\" with { \"resolution-mode\": \"import\" };\n\
                 import { RequireInterface as Req } from \"pkg\";\n",
        ),
    ];
    let codes = check_files(&files, "/index.ts", |o| {
        o.module = ModuleKind::Node16;
        o.module_resolution = ModuleResolutionKind::Node16;
    });
    assert_eq!(
        codes,
        vec![2305, 2823],
        "type-only override resolves the import face (clean); the plain clause \
             takes the default CJS chain → ImportInterface missing (2305) + TS2823 \
             for the attribute on node16"
    );
}

#[test]
fn types_option_symbols_resolve_before_declaring_file_checked() {
    let files = [
        (
            "/types/jquery/index.d.ts",
            "declare var $: { foo(): void };\n",
        ),
        ("/index.ts", "const q: number = $;\n$.nope();\n$.foo();\n"),
    ];
    let codes = check_files(&files, "/index.ts", |o| {
        o.types = vec!["jquery".to_string()];
        o.type_roots = vec!["/types".to_string()];
    });
    assert_eq!(
        codes,
        vec![2322, 2339],
        "$ must be typed from the auto-included d.ts (2322 for `q: number = $`, \
             2339 for the missing member) — not silently any"
    );
}

#[test]
fn jsx_runtime_import_source_unresolvable_reports_2875() {
    let files = [
        (
            "/lib.d.ts",
            "declare namespace JSX { interface Element {} }\n",
        ),
        ("/index.tsx", "const a = <div />;\nexport {};\n"),
    ];
    let codes = check_files(&files, "/index.tsx", |o| {
        o.jsx = crate::core::compiler_options::JsxEmit::ReactJSX;
        o.jsx_import_source = "preact".to_string();
    });
    assert_eq!(
        codes,
        vec![2875],
        "unresolvable jsx runtime must report TS2875"
    );

    let codes = check_files(&files, "/index.tsx", |o| {
        o.jsx = crate::core::compiler_options::JsxEmit::React;
        o.jsx_import_source = "preact".to_string();
    });
    assert_eq!(
        codes,
        vec![2874],
        "classic mode reports no TS2875 but TS2874 without React in scope"
    );
}

#[test]
fn namespace_import_alias_qualified_type_access() {
    let files = [
        (
            "/amb.d.ts",
            "declare module \"pkg\" {\n    export type VM<T> = { [K in keyof T]-?: number };\n}\ndeclare module \"outer\" {\n    import * as P from \"pkg\";\n    namespace Inner {\n        type Alias<T> = P.VM<T>;\n    }\n    export = Inner;\n}\n",
        ),
        (
            "/index.ts",
            "/// <reference path=\"/amb.d.ts\" />\nimport * as O from \"outer\";\nexport declare const y: O.Alias<{}>;\n",
        ),
    ];
    let codes = check_files(&files, "/index.ts", |o| {
        o.target = crate::core::compiler_options::ScriptTarget::ES2015;
    });
    assert_eq!(
        codes,
        Vec::<i32>::new(),
        "ambient-module namespace-import qualified access must resolve"
    );
}

#[test]
fn generic_callback_reference_infers_no_2345() {
    let codes = check_files(
        &[(
            "/index.ts",
            "function identity<A>(a: A): A { return a; }\nconst x = [1, 2, 3].map(identity)[0];\nexport {};\n",
        )],
        "/index.ts",
        |o| {
            o.target = crate::core::compiler_options::ScriptTarget::ES2015;
        },
    );
    assert_eq!(
        codes,
        Vec::<i32>::new(),
        "map(identity) must not report TS2345"
    );
}

#[test]
fn explicit_node10_resolution_ignores_exports() {
    let files = [
        (
            "/node_modules/pkg/package.json",
            r#"{ "name": "pkg", "version": "1.0.0", "exports": { ".": "./definitely-not-index.js" } }"#,
        ),
        (
            "/node_modules/pkg/definitely-not-index.d.ts",
            "export {};\n",
        ),
        ("/index.ts", "import { pkg } from \"pkg\";\n"),
    ];
    let codes = check_files(&files, "/index.ts", |o| {
        o.module_resolution = ModuleResolutionKind::Node10;
        o.target = crate::core::compiler_options::ScriptTarget::ES2015;
    });
    assert_eq!(codes, vec![2307], "node10 must not resolve via exports");
}

#[test]
fn dom_two_level_heritage_assignable_no_phantom_2739() {
    let codes = check_files(
        &[(
            "/index.ts",
            "declare const h: HTMLElement;\nconst e: Element = h;\nexport {};\n",
        )],
        "/index.ts",
        |o| {
            o.target = crate::core::compiler_options::ScriptTarget::ES2022;
        },
    );
    assert_eq!(
        codes,
        Vec::<i32>::new(),
        "two-level DOM heritage must be assignable"
    );
}

#[test]
fn reserved_cjs_top_level_names() {
    let body = "function require() {}\n\
                    const exports = {};\n\
                    class Object {}\n\
                    export const __esModule = false;\n\
                    export {require, exports, Object};\n";
    let files = [
        (
            "/proj/package.json",
            r#"{"name": "package", "type": "module"}"#,
        ),
        ("/proj/sub/package.json", r#"{"type": "commonjs"}"#),
        ("/proj/sub/index.ts", body),
        ("/proj/index.ts", body),
    ];
    let cjs = check_files(&files, "/proj/sub/index.ts", |o| {
        o.module = ModuleKind::Node16;
        o.module_resolution = ModuleResolutionKind::Node16;
    });
    assert_eq!(cjs, vec![2441, 2441, 2725, 1216], "CJS file reserved names");
    let esm = check_files(&files, "/proj/index.ts", |o| {
        o.module = ModuleKind::Node16;
        o.module_resolution = ModuleResolutionKind::Node16;
    });
    assert_eq!(esm, Vec::<i32>::new(), "ESM file has no collisions");
}

#[test]
fn import_attributes_2823_suppressed_on_parse_error() {
    let codes = check_files(
        &[("/proj/index.ts", "import * as f from \"./first\" with {\n")],
        "/proj/index.ts",
        |o| {
            o.module = ModuleKind::CommonJS;
        },
    );
    assert!(
        !codes.contains(&2823),
        "TS2823 must be suppressed on files with parse errors: {codes:?}"
    );
    assert!(codes.contains(&1005), "expected the parse error: {codes:?}");
}

#[test]
fn type_only_resolution_mode_attribute_grammar() {
    let files = [
        (
            "/proj/node_modules/pkg/package.json",
            r#"{"name": "pkg", "exports": {"import": "./import.js", "require": "./require.js"}}"#,
        ),
        (
            "/proj/node_modules/pkg/import.d.ts",
            "export interface ImportInterface {}\n",
        ),
        (
            "/proj/node_modules/pkg/require.d.ts",
            "export interface RequireInterface {}\n",
        ),
        (
            "/proj/index.ts",
            "import type { RequireInterface } from \"pkg\" with { \"resolution-mode\": \"require\" };\n\
                 import { ImportInterface } from \"pkg\" with { \"resolution-mode\": \"import\" };\n\
                 export interface L extends RequireInterface {}\n",
        ),
    ];
    let codes = check_files(&files, "/proj/index.ts", |o| {
        o.module = ModuleKind::Node16;
        o.module_resolution = ModuleResolutionKind::Node16;
    });

    assert_eq!(
        codes.iter().filter(|c| **c == 2823).count(),
        1,
        "one TS2823 for the non-type-only clause: {codes:?}"
    );

    let bad = check_files(
        &[(
            "/proj/index.ts",
            "import type { X } from \"./missing\" with { \"resolution-mode\": \"foobar\" };\n",
        )],
        "/proj/index.ts",
        |o| {
            o.module = ModuleKind::Node18;
            o.module_resolution = ModuleResolutionKind::Node16;
        },
    );
    assert!(bad.contains(&1453), "bad resolution-mode value: {bad:?}");
}

#[test]
fn overload_probe_does_not_leak_diagnostics() {
    let codes = check_files(
        &[(
            "/proj/index.ts",
            "var fa: number[];\nfa = fa.concat([0]);\n",
        )],
        "/proj/index.ts",
        |_| {},
    );
    assert_eq!(codes, vec![2454], "only used-before-assigned: {codes:?}");
}

#[test]
fn generic_arity_error_suppresses_ts2564() {
    let codes = check_files(
        &[(
            "/proj/index.ts",
            "export interface A<T> {\n   new (dbSet: DbSet<T>): T;\n}\n\
                 export class DbSet<T> {\n    _entityType: A;\n  get entityType() { return this._entityType; }\n}\n",
        )],
        "/proj/index.ts",
        |o| {
            o.module = ModuleKind::CommonJS;
        },
    );
    assert_eq!(
        codes.iter().filter(|c| **c == 2314).count(),
        1,
        "exactly one TS2314: {codes:?}"
    );
    assert!(
        !codes.contains(&2564),
        "TS2564 must be suppressed by the error-typed annotation: {codes:?}"
    );
}

#[test]
fn ambient_declarations_exempt_from_reserved_names() {
    let codes = check_files(
        &[(
            "/proj/index.ts",
            "export declare var exports: number;\n\
                 export declare var require: string;\n\
                 declare namespace inner { var exports: string; }\n",
        )],
        "/proj/index.ts",
        |o| {
            o.module = ModuleKind::CommonJS;
        },
    );
    assert_eq!(
        codes,
        Vec::<i32>::new(),
        "ambient names are clean: {codes:?}"
    );
}

#[test]
fn es_module_marker_requires_export_and_emit() {
    let bare = check_files(
        &[(
            "/proj/index.ts",
            "export default \"test\";\nvar __esModule = 1;\n",
        )],
        "/proj/index.ts",
        |o| {
            o.module = ModuleKind::CommonJS;
        },
    );
    assert_eq!(
        bare,
        Vec::<i32>::new(),
        "bare __esModule is legal: {bare:?}"
    );

    let exported = check_files(
        &[(
            "/proj/index.ts",
            "export default \"test\";\nexport var __esModule = 1;\n",
        )],
        "/proj/index.ts",
        |o| {
            o.module = ModuleKind::CommonJS;
        },
    );
    assert_eq!(
        exported,
        vec![1216],
        "exported __esModule reports TS1216: {exported:?}"
    );

    let noemit = check_files(
        &[(
            "/proj/index.ts",
            "export default \"test\";\nexport var __esModule = 1;\n",
        )],
        "/proj/index.ts",
        |o| {
            o.module = ModuleKind::CommonJS;
            o.no_emit = crate::core::tristate::Tristate::True;
        },
    );
    assert_eq!(
        noemit,
        Vec::<i32>::new(),
        "noEmit skips the marker check: {noemit:?}"
    );
}
