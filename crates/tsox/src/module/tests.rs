use super::*;

#[test]
fn parse_package_name_simple() {
    let (pkg, rest) = parse_package_name("foo");
    assert_eq!(pkg, "foo");
    assert_eq!(rest, "");
}

#[test]
fn parse_package_name_with_subpath() {
    let (pkg, rest) = parse_package_name("foo/bar");
    assert_eq!(pkg, "foo");
    assert_eq!(rest, "bar");
}

#[test]
fn parse_package_name_scoped() {
    let (pkg, rest) = parse_package_name("@scope/name");
    assert_eq!(pkg, "@scope/name");
    assert_eq!(rest, "");
}

#[test]
fn parse_package_name_scoped_with_subpath() {
    let (pkg, rest) = parse_package_name("@scope/name/sub");
    assert_eq!(pkg, "@scope/name");
    assert_eq!(rest, "sub");
}

#[test]
fn mangle_scoped_package() {
    assert_eq!(mangle_scoped_package_name("@scope/name"), "scope__name");
    assert_eq!(mangle_scoped_package_name("foo"), "foo");
}

#[test]
fn unmangle_scoped_package() {
    assert_eq!(unmangle_scoped_package_name("scope__name"), "@scope/name");
    assert_eq!(unmangle_scoped_package_name("foo"), "foo");
}

#[test]
fn types_package_name() {
    assert_eq!(get_types_package_name("foo"), "@types/foo");
    assert_eq!(get_types_package_name("@scope/name"), "@types/scope__name");
}

#[test]
fn package_name_from_types() {
    assert_eq!(
        get_package_name_from_types_package_name("@types/foo"),
        "foo"
    );
    assert_eq!(
        get_package_name_from_types_package_name("@types/scope__name"),
        "@scope/name"
    );
}

#[test]
fn parse_node_module_from_path() {
    let cases: &[(&str, &str, bool, &str)] = &[
        (
            "file in package",
            "/a/node_modules/b/lib/index.d.ts",
            false,
            "/a/node_modules/b",
        ),
        (
            "file in scoped package",
            "/a/node_modules/@scope/b/lib/index.d.ts",
            false,
            "/a/node_modules/@scope/b",
        ),
        (
            "folder subpath",
            "/a/node_modules/b/lib/File",
            true,
            "/a/node_modules/b",
        ),
        (
            "folder subpath scoped",
            "/a/node_modules/@scope/b/lib/File",
            true,
            "/a/node_modules/@scope/b",
        ),
        (
            "package root folder",
            "/a/node_modules/b",
            true,
            "/a/node_modules/b",
        ),
        (
            "scoped package root folder",
            "/a/node_modules/@scope/b",
            true,
            "/a/node_modules/@scope/b",
        ),
        (
            "scope-only folder",
            "/a/node_modules/@scope",
            true,
            "/a/node_modules/@scope",
        ),
        (
            "types scope-only folder",
            "/a/node_modules/@types",
            true,
            "/a/node_modules/@types",
        ),
        ("not in node_modules", "/a/src/index.ts", false, ""),
    ];
    for (name, path, is_folder, want) in cases {
        let got = super::parse_node_module_from_path(path, *is_folder);
        assert_eq!(
            got, *want,
            "parse_node_module_from_path({path:?}, {is_folder})"
        );
        let _ = name;
    }
}

#[test]
fn resolve_module_name_trailing_slash() {
    use crate::core::compiler_options::{
        CompilerOptions, ModuleKind, ModuleResolutionKind, ScriptTarget,
    };
    use crate::vfs::{FS, InMemoryFS};
    use std::sync::Arc;

    struct ResolutionHostStub {
        fs: InMemoryFS,
        cwd: String,
    }
    impl ResolutionHost for ResolutionHostStub {
        fn fs(&self) -> &dyn FS {
            &self.fs
        }
        fn get_current_directory(&self) -> &str {
            &self.cwd
        }
    }

    let fs = InMemoryFS::new();
    for dir in [
        "/repo",
        "/repo/src",
        "/repo/node_modules",
        "/repo/node_modules/pkg",
    ] {
        fs.insert_dir(dir);
    }
    fs.write_file(
        "/repo/node_modules/pkg/package.json",
        r#"{"name":"pkg","main":"main.js","types":"main.d.ts"}"#,
    )
    .unwrap();
    fs.write_file(
        "/repo/node_modules/pkg/main.d.ts",
        "export const x: number;",
    )
    .unwrap();
    fs.write_file("/repo/node_modules/pkg/main.js", "exports.x = 1;")
        .unwrap();
    fs.write_file("/repo/src/file.ts", "").unwrap();

    let host = Arc::new(ResolutionHostStub {
        fs,
        cwd: "/repo".to_string(),
    });
    let mut opts = CompilerOptions::default();
    opts.module_resolution = ModuleResolutionKind::Bundler;
    opts.module = ModuleKind::ESNext;
    opts.target = ScriptTarget::ESNext;
    let resolver = Resolver::new(host, Arc::new(opts), String::new(), String::new());

    for name in ["pkg", "pkg/"] {
        let (resolved, _) =
            resolver.resolve_module_name(name, "/repo/src/file.ts", ModuleKind::ESNext, None);
        assert!(
            resolved.as_ref().is_some_and(|r| r.is_resolved()),
            "{name:?} failed to resolve"
        );
    }
}

#[test]
fn resolve_module_name_trailing_slash_race() {
    use crate::core::compiler_options::ModuleKind;
    use std::thread;

    const N: usize = 8;
    let resolver = build_concurrent_resolver();

    let results = thread::scope(|s| {
        (0..N)
            .map(|i| {
                let name = if i % 2 == 0 { "pkg" } else { "pkg/" };
                let r = &resolver;
                s.spawn(move || {
                    let (resolved, _) =
                        r.resolve_module_name(name, "/repo/src/file.ts", ModuleKind::ESNext, None);
                    (name, resolved)
                })
            })
            .map(|h| h.join().unwrap())
            .collect::<Vec<_>>()
    });

    for (name, r) in &results {
        assert!(
            r.as_ref().is_some_and(|m| m.is_resolved()),
            "{name:?} failed to resolve"
        );
    }

    let expected = results[0].1.as_ref().unwrap().resolved_file_name.clone();
    for (name, r) in &results {
        assert_eq!(
            r.as_ref().unwrap().resolved_file_name,
            expected,
            "{name:?} resolved to a different file than the first thread"
        );
    }
}

fn build_concurrent_resolver() -> Resolver {
    use crate::core::compiler_options::{
        CompilerOptions, ModuleKind, ModuleResolutionKind, ScriptTarget,
    };
    use crate::vfs::{FS, InMemoryFS};
    use std::sync::Arc;

    struct ResolutionHostStub {
        fs: InMemoryFS,
        cwd: String,
    }
    impl ResolutionHost for ResolutionHostStub {
        fn fs(&self) -> &dyn FS {
            &self.fs
        }
        fn get_current_directory(&self) -> &str {
            &self.cwd
        }
    }

    let fs = InMemoryFS::new();
    for dir in [
        "/repo",
        "/repo/src",
        "/repo/node_modules",
        "/repo/node_modules/pkg",
    ] {
        fs.insert_dir(dir);
    }
    fs.write_file(
        "/repo/node_modules/pkg/package.json",
        r#"{"name":"pkg","main":"main.js","types":"main.d.ts"}"#,
    )
    .unwrap();
    fs.write_file(
        "/repo/node_modules/pkg/main.d.ts",
        "export const x: number;",
    )
    .unwrap();
    fs.write_file("/repo/node_modules/pkg/main.js", "exports.x = 1;")
        .unwrap();
    fs.write_file("/repo/src/file.ts", "").unwrap();

    let host = Arc::new(ResolutionHostStub {
        fs,
        cwd: "/repo".to_string(),
    });
    let mut opts = CompilerOptions::default();
    opts.module_resolution = ModuleResolutionKind::Bundler;
    opts.module = ModuleKind::ESNext;
    opts.target = ScriptTarget::ESNext;
    Resolver::new(host, Arc::new(opts), String::new(), String::new())
}

#[test]
fn resolve_subpath_nil_contents_race() {
    use crate::core::compiler_options::ModuleKind;
    use std::thread;

    const N: usize = 8;
    let resolver = build_concurrent_resolver();

    let results = thread::scope(|s| {
        (0..N)
            .map(|_| {
                s.spawn(|| {
                    let (resolved, _) = resolver.resolve_module_name(
                        "pkg",
                        "/repo/src/file.ts",
                        ModuleKind::ESNext,
                        None,
                    );
                    resolved
                })
            })
            .map(|h| h.join().unwrap())
            .collect::<Vec<_>>()
    });

    assert!(
        results
            .iter()
            .all(|r| r.as_ref().is_some_and(|m| m.is_resolved()))
    );
    let expected = results[0].as_ref().unwrap().resolved_file_name.clone();
    assert!(
        results
            .iter()
            .all(|r| r.as_ref().unwrap().resolved_file_name == expected),
        "threads disagreed on resolved file name: {results:?}"
    );
}

#[test]
fn resolve_peer_dependency_nil_contents_race() {
    use crate::core::compiler_options::ModuleKind;
    use std::thread;

    const N: usize = 8;
    let resolver = build_concurrent_resolver();

    let results = thread::scope(|s| {
        (0..N)
            .flat_map(|i| {
                let name = if i % 2 == 0 { "pkg" } else { "pkg/" };
                let r = &resolver;
                Some(s.spawn(move || {
                    let (resolved, _) =
                        r.resolve_module_name(name, "/repo/src/file.ts", ModuleKind::ESNext, None);
                    resolved
                }))
            })
            .map(|h| h.join().unwrap())
            .collect::<Vec<_>>()
    });

    assert!(
        results
            .iter()
            .all(|r| r.as_ref().is_some_and(|m| m.is_resolved()))
    );
    let expected = results[0].as_ref().unwrap().resolved_file_name.clone();
    assert!(
        results
            .iter()
            .all(|r| r.as_ref().unwrap().resolved_file_name == expected),
        "threads disagreed on resolved file name: {results:?}"
    );
}
