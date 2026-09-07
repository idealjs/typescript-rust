use super::*;

#[test]
fn extensions_bitfield() {
    let ts = Extensions::TYPESCRIPT;
    assert!(ts.contains(Extensions::TYPESCRIPT));
    assert!(!ts.contains(Extensions::JAVASCRIPT));

    let both = Extensions::TYPESCRIPT.union(Extensions::JAVASCRIPT);
    assert_eq!(both, Extensions::IMPLEMENTATION_FILES);

    let all = Extensions::TYPESCRIPT
        .union(Extensions::JAVASCRIPT)
        .union(Extensions::DECLARATION)
        .union(Extensions::JSON);
    assert_eq!(all.bits(), 0b1111);
}

#[test]
fn extensions_array() {
    let ts = Extensions::TYPESCRIPT;
    let arr = ts.array();
    assert!(arr.contains(&".ts"));
    assert!(arr.contains(&".tsx"));

    let decl = Extensions::DECLARATION;
    let arr = decl.array();
    assert!(arr.contains(&".d.ts"));
}

#[test]
fn extensions_string() {
    let both = Extensions::TYPESCRIPT.union(Extensions::JAVASCRIPT);
    assert_eq!(both.extensions_string(), "TypeScript, JavaScript");
}

#[test]
fn module_cache_first_writer_wins() {
    let cache = ModuleResolutionCache::new();
    let key = ModuleResolutionCacheKey {
        containing_directory: "/foo".to_string(),
        module_name: "bar".to_string(),
        resolution_mode: ModuleKind::None,
        redirect_config_name: String::new(),
    };
    let mod1 = Arc::new(ResolvedModule {
        resolved_file_name: "/foo/bar1.ts".to_string(),
        ..Default::default()
    });
    let mod2 = Arc::new(ResolvedModule {
        resolved_file_name: "/foo/bar2.ts".to_string(),
        ..Default::default()
    });
    cache.set(key.clone(), mod1);
    cache.set(key.clone(), mod2);
    let result = cache.get(&key).unwrap();
    assert_eq!(result.resolved_file_name, "/foo/bar1.ts");
}

#[test]
fn type_ref_cache_last_writer_wins() {
    let cache = TypeRefDirectiveResolutionCache::new();
    let key = TypeRefDirectiveCacheKey {
        containing_directory: "/foo".to_string(),
        type_reference_name: "node".to_string(),
        resolution_mode: ModuleKind::None,
        redirect_config_name: String::new(),
        from_inferred_types_containing_file: false,
    };
    let dir1 = Arc::new(ResolvedTypeReferenceDirective {
        resolved_file_name: "/foo/node1.d.ts".to_string(),
        ..Default::default()
    });
    let dir2 = Arc::new(ResolvedTypeReferenceDirective {
        resolved_file_name: "/foo/node2.d.ts".to_string(),
        ..Default::default()
    });
    cache.set(key.clone(), dir1);
    cache.set(key.clone(), dir2);
    let result = cache.get(&key).unwrap();
    assert_eq!(result.resolved_file_name, "/foo/node2.d.ts");
}

#[test]
fn effective_type_roots_default() {
    let opts = CompilerOptions::default();
    let (roots, from_config) = get_effective_type_roots(&opts, "/project/sub");
    assert!(!from_config);

    assert_eq!(roots.len(), 3);
    assert!(roots[0].contains("sub/node_modules/@types"));
    assert!(roots[1].contains("project/node_modules/@types"));

    assert_eq!(roots[2], "/node_modules/@types");
}

#[test]
fn effective_type_roots_explicit() {
    let mut opts = CompilerOptions::default();
    opts.type_roots = vec!["./custom-types".to_string()];
    let (roots, from_config) = get_effective_type_roots(&opts, "/project");
    assert!(from_config);
    assert_eq!(roots, vec!["./custom-types".to_string()]);
}

#[test]
fn effective_type_roots_base_on_config_file() {
    let mut opts = CompilerOptions::default();
    opts.config_file_path = "/foo/bar/tsconfig.json".to_string();
    let (roots, from_config) = get_effective_type_roots(&opts, "/src");
    assert!(!from_config);
    assert_eq!(roots.len(), 3);
    assert_eq!(roots[0], "/foo/bar/node_modules/@types");
    assert_eq!(roots[1], "/foo/node_modules/@types");
    assert_eq!(roots[2], "/node_modules/@types");
}

fn make_state<'a>(
    name: &str,
    containing_dir: &str,
    opts: &'a CompilerOptions,
    fs: &'a dyn FS,
) -> ResolutionState<'a> {
    ResolutionState::new(name, containing_dir, false, ModuleKind::None, opts, fs, "/")
}

const REL_EXTS: Extensions = Extensions::TYPESCRIPT
    .union(Extensions::JAVASCRIPT)
    .union(Extensions::DECLARATION);

#[test]
fn resolve_relative_ts_file() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");
    fs.write_file("/src/foo.ts", "export const x = 1;").unwrap();

    let opts = CompilerOptions::default();
    let mut state = make_state("./foo", "/src", &opts, &fs);
    let candidate = ResolutionState::normalize_path_for_cjs_resolution("/src", "./foo");
    let result = state.node_load_module_by_relative_name(REL_EXTS, &candidate, true);
    assert!(result.is_some());
    let resolved = result.unwrap();
    assert_eq!(resolved.path, "/src/foo.ts");
    assert_eq!(resolved.extension, ".ts");
}

#[test]
fn resolve_relative_tsx_file() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");
    fs.write_file("/src/component.tsx", "export const C = 1;")
        .unwrap();

    let opts = CompilerOptions::default();
    let mut state = make_state("./component", "/src", &opts, &fs);
    let candidate = ResolutionState::normalize_path_for_cjs_resolution("/src", "./component");
    let result = state.node_load_module_by_relative_name(REL_EXTS, &candidate, true);
    assert!(result.is_some());
    assert_eq!(result.unwrap().extension, ".tsx");
}

#[test]
fn resolve_relative_js_specifier_swaps_to_ts() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");

    fs.write_file("/src/foo.ts", "export const x = 1;").unwrap();

    let opts = CompilerOptions::default();
    let mut state = make_state("./foo.js", "/src", &opts, &fs);
    let candidate = ResolutionState::normalize_path_for_cjs_resolution("/src", "./foo.js");
    let result = state.node_load_module_by_relative_name(REL_EXTS, &candidate, true);
    assert!(result.is_some());
    assert_eq!(result.unwrap().path, "/src/foo.ts");
}

#[test]
fn resolve_relative_mjs_specifier_swaps_to_mts() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");
    fs.write_file("/src/foo.mts", "export const x = 1;")
        .unwrap();

    let opts = CompilerOptions::default();
    let mut state = make_state("./foo.mjs", "/src", &opts, &fs);
    let candidate = ResolutionState::normalize_path_for_cjs_resolution("/src", "./foo.mjs");
    let result = state.node_load_module_by_relative_name(REL_EXTS, &candidate, true);
    assert!(result.is_some());
    assert_eq!(result.unwrap().path, "/src/foo.mts");
}

#[test]
fn resolve_relative_nonexistent_returns_none() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");

    let opts = CompilerOptions::default();
    let mut state = make_state("./missing", "/src", &opts, &fs);
    let candidate = ResolutionState::normalize_path_for_cjs_resolution("/src", "./missing");
    let result = state.node_load_module_by_relative_name(REL_EXTS, &candidate, true);
    assert!(result.is_none());
}

#[test]
fn exports_target_nesting_bounded() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/node_modules/pkg");
    fs.write_file("/node_modules/pkg/index.ts", "export const x = 1;")
        .unwrap();

    let opts = CompilerOptions::default();

    let shallow = r#"{"name": "pkg", "exports": {"default": {"default": "./index.ts"}}}"#;
    let fields = packagejson::parse(shallow).unwrap();
    let mut state = make_state("pkg", "/src", &opts, &fs);
    let resolved = state.load_module_from_exports(
        REL_EXTS,
        ".",
        "/node_modules/pkg",
        &fields.path_fields.exports,
    );
    assert_eq!(resolved.unwrap().path, "/node_modules/pkg/index.ts");

    let mut target = r#""./index.ts""#.to_string();
    for _ in 0..30 {
        target = format!(r#"{{"default": {target}}}"#);
    }
    let deep = format!(r#"{{"name": "pkg", "exports": {target}}}"#);
    let fields = packagejson::parse(&deep).unwrap();
    let mut state = make_state("pkg", "/src", &opts, &fs);
    let result = state.load_module_from_exports(
        REL_EXTS,
        ".",
        "/node_modules/pkg",
        &fields.path_fields.exports,
    );
    assert!(
        result.is_none(),
        "deeply nested exports must stop at the cap"
    );
}

#[test]
fn resolve_relative_parent_dir_not_exists() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();

    let opts = CompilerOptions::default();
    let mut state = make_state("./foo", "/nonexistent", &opts, &fs);
    let candidate = ResolutionState::normalize_path_for_cjs_resolution("/nonexistent", "./foo");
    let result = state.node_load_module_by_relative_name(Extensions::TYPESCRIPT, &candidate, true);
    assert!(result.is_none());
}

#[test]
fn normalize_path_for_dot() {
    let result = ResolutionState::normalize_path_for_cjs_resolution("/src", ".");
    assert!(result.ends_with('/'));
    assert!(tspath::has_trailing_directory_separator(&result));
}

#[test]
fn normalize_path_for_dot_dot() {
    let result = ResolutionState::normalize_path_for_cjs_resolution("/src", "..");
    assert!(tspath::has_trailing_directory_separator(&result));
}

#[test]
fn resolve_bare_specifier_node_modules() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");
    fs.insert_dir("/src/node_modules");
    fs.insert_dir("/src/node_modules/foo");
    fs.write_file("/src/node_modules/foo/index.ts", "export const x = 1;")
        .unwrap();

    let opts = CompilerOptions::default();
    let state = make_state("foo", "/src", &opts, &fs);
    let result = state.resolve_node_like();
    assert!(result.is_resolved());
    assert_eq!(result.resolved_file_name, "/src/node_modules/foo/index.ts");
    assert!(result.is_external_library_import);
}

#[test]
fn resolve_bare_specifier_with_types() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");
    fs.insert_dir("/src/node_modules");
    fs.insert_dir("/src/node_modules/foo");
    fs.insert_dir("/src/node_modules/foo/dist");
    fs.write_file(
        "/src/node_modules/foo/package.json",
        r#"{"name": "foo", "types": "./dist/index.d.ts"}"#,
    )
    .unwrap();
    fs.write_file(
        "/src/node_modules/foo/dist/index.d.ts",
        "export const x = 1;",
    )
    .unwrap();

    let opts = CompilerOptions::default();
    let state = make_state("foo", "/src", &opts, &fs);
    let result = state.resolve_node_like();
    assert!(result.is_resolved());
    assert_eq!(
        result.resolved_file_name,
        "/src/node_modules/foo/dist/index.d.ts"
    );
}

#[test]
fn resolve_bare_specifier_with_main() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");
    fs.insert_dir("/src/node_modules");
    fs.insert_dir("/src/node_modules/foo");
    fs.insert_dir("/src/node_modules/foo/lib");
    fs.write_file(
        "/src/node_modules/foo/package.json",
        r#"{"name": "foo", "main": "./lib/index.js"}"#,
    )
    .unwrap();
    fs.write_file("/src/node_modules/foo/lib/index.js", "exports.x = 1;")
        .unwrap();

    let opts = CompilerOptions::default();
    let state = make_state("foo", "/src", &opts, &fs);
    let result = state.resolve_node_like();
    assert!(result.is_resolved());
    assert_eq!(
        result.resolved_file_name,
        "/src/node_modules/foo/lib/index.js"
    );
}

#[test]
fn resolve_bare_specifier_ancestor_node_modules() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");
    fs.insert_dir("/src/sub");
    fs.insert_dir("/node_modules");
    fs.insert_dir("/node_modules/foo");
    fs.write_file("/node_modules/foo/index.ts", "export const x = 1;")
        .unwrap();

    let opts = CompilerOptions::default();
    let state = make_state("foo", "/src/sub", &opts, &fs);
    let result = state.resolve_node_like();
    assert!(result.is_resolved());
    assert_eq!(result.resolved_file_name, "/node_modules/foo/index.ts");
}

#[test]
fn resolve_bare_specifier_not_found() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");
    fs.insert_dir("/src/node_modules");

    let opts = CompilerOptions::default();
    let state = make_state("nonexistent", "/src", &opts, &fs);
    let result = state.resolve_node_like();
    assert!(!result.is_resolved());
    assert!(result.resolved_file_name.is_empty());
}

#[test]
fn node16_conditions_follow_resolution_mode() {
    let mut opts = CompilerOptions::default();
    opts.module_resolution = ModuleResolutionKind::Node16;
    let require = get_conditions(&opts, ModuleKind::CommonJS);
    assert!(require.contains(&"require".to_string()));
    assert!(!require.contains(&"import".to_string()));
    let import = get_conditions(&opts, ModuleKind::ESNext);
    assert!(import.contains(&"import".to_string()));
    assert!(!import.contains(&"require".to_string()));

    for c in [&require, &import] {
        assert!(c.contains(&"node".to_string()));
        assert!(c.contains(&"types".to_string()));
    }
}

#[test]
fn node16_exports_condition_by_file_format() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    for d in ["/proj", "/proj/sub", "/proj/node_modules/pkg"] {
        fs.insert_dir(d);
    }
    fs.insert_file(
        "/proj/package.json",
        r#"{"name": "root", "type": "module"}"#,
    );
    fs.insert_file("/proj/sub/package.json", r#"{"type": "commonjs"}"#);
    fs.insert_file(
        "/proj/node_modules/pkg/package.json",
        r#"{"name": "pkg", "exports": {"import": "./import.js", "require": "./require.js"}}"#,
    );
    fs.insert_file("/proj/node_modules/pkg/import.d.ts", "export {};\n");
    fs.insert_file("/proj/node_modules/pkg/require.d.ts", "export {};\n");
    fs.insert_file("/proj/index.ts", "import \"pkg\";\n");
    fs.insert_file("/proj/sub/index.ts", "import \"pkg\";\n");

    let mut opts = CompilerOptions::default();
    opts.module_resolution = ModuleResolutionKind::Node16;

    assert_eq!(
        default_resolution_mode(ModuleKind::None, &opts, "/proj/index.ts", &fs),
        ModuleKind::ESNext
    );
    assert_eq!(
        default_resolution_mode(ModuleKind::None, &opts, "/proj/sub/index.ts", &fs),
        ModuleKind::CommonJS
    );

    let esm = ResolutionState::new(
        "pkg",
        "/proj",
        false,
        ModuleKind::ESNext,
        &opts,
        &fs,
        "/proj",
    );
    let r = esm.resolve_node_like();
    assert!(r.is_resolved(), "esm resolve");
    assert!(
        r.resolved_file_name.ends_with("import.d.ts"),
        "{}",
        r.resolved_file_name
    );

    let cjs = ResolutionState::new(
        "pkg",
        "/proj/sub",
        false,
        ModuleKind::CommonJS,
        &opts,
        &fs,
        "/proj",
    );
    let r = cjs.resolve_node_like();
    assert!(r.is_resolved(), "cjs resolve");
    assert!(
        r.resolved_file_name.ends_with("require.d.ts"),
        "{}",
        r.resolved_file_name
    );
}

#[test]
fn implied_format_from_package_json_chain() {
    use crate::core::compiler_options::ModuleKind;
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    for d in ["/a/b", "/a/node_modules"] {
        fs.insert_dir(d);
    }
    fs.insert_file("/a/package.json", r#"{"type": "module"}"#);
    let read = |p: &str| fs.read_file(p);
    assert_eq!(
        crate::compiler::implied_node_format_of_file("/a/b/x.ts", &read),
        ModuleKind::ESNext
    );
    assert_eq!(
        crate::compiler::implied_node_format_of_file("/a/b/x.mts", &read),
        ModuleKind::ESNext
    );
    assert_eq!(
        crate::compiler::implied_node_format_of_file("/a/b/x.cts", &read),
        ModuleKind::CommonJS
    );

    assert_eq!(
        crate::compiler::implied_node_format_of_file("/a/node_modules/x.ts", &read),
        ModuleKind::ESNext
    );
}

#[test]
fn resolve_types_fallback() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");
    fs.insert_dir("/src/node_modules");
    fs.insert_dir("/src/node_modules/@types");
    fs.insert_dir("/src/node_modules/@types/foo");
    fs.write_file(
        "/src/node_modules/@types/foo/index.d.ts",
        "declare const x: number;",
    )
    .unwrap();

    let opts = CompilerOptions::default();
    let state = make_state("foo", "/src", &opts, &fs);
    let result = state.resolve_node_like();
    assert!(result.is_resolved());
    assert_eq!(
        result.resolved_file_name,
        "/src/node_modules/@types/foo/index.d.ts"
    );
}

#[test]
fn resolve_paths_exact_match() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");
    fs.insert_dir("/src/mapped");
    fs.write_file("/src/mapped/foo.ts", "export const x = 1;")
        .unwrap();

    let mut opts = CompilerOptions::default();
    opts.paths = Some({
        let mut m = std::collections::HashMap::new();
        m.insert("foo".to_string(), vec!["./mapped/foo".to_string()]);
        m
    });
    opts.paths_base_path = "/src".to_string();

    let state = ResolutionState::new("foo", "/src", false, ModuleKind::None, &opts, &fs, "/src");
    let result = state.resolve_node_like();
    assert!(result.is_resolved());
    assert_eq!(result.resolved_file_name, "/src/mapped/foo.ts");
}

#[test]
fn resolve_paths_wildcard() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");
    fs.insert_dir("/src/types");
    fs.write_file("/src/types/bar.ts", "export const x = 1;")
        .unwrap();

    let mut opts = CompilerOptions::default();
    opts.paths = Some({
        let mut m = std::collections::HashMap::new();
        m.insert("@mytypes/*".to_string(), vec!["./types/*".to_string()]);
        m
    });
    opts.paths_base_path = "/src".to_string();

    let state = ResolutionState::new(
        "@mytypes/bar",
        "/src",
        false,
        ModuleKind::None,
        &opts,
        &fs,
        "/src",
    );
    let result = state.resolve_node_like();
    assert!(result.is_resolved());
    assert_eq!(result.resolved_file_name, "/src/types/bar.ts");
}

#[test]
fn resolve_paths_no_match_falls_through() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");

    let mut opts = CompilerOptions::default();
    opts.paths = Some({
        let mut m = std::collections::HashMap::new();
        m.insert("foo".to_string(), vec!["./mapped/foo".to_string()]);
        m
    });
    opts.paths_base_path = "/src".to_string();

    let state = ResolutionState::new("bar", "/src", false, ModuleKind::None, &opts, &fs, "/src");
    let result = state.resolve_node_like();
    assert!(!result.is_resolved());
}

#[test]
fn resolve_root_dirs() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src/generated");
    fs.insert_dir("/src/manual");
    fs.write_file("/src/manual/shared.ts", "export const x = 1;")
        .unwrap();

    let mut opts = CompilerOptions::default();
    opts.root_dirs = vec!["/src/generated".to_string(), "/src/manual".to_string()];

    let state = ResolutionState::new(
        "./shared",
        "/src/generated",
        false,
        ModuleKind::None,
        &opts,
        &fs,
        "/src",
    );
    let result = state.resolve_node_like();
    assert!(result.is_resolved());
    assert_eq!(result.resolved_file_name, "/src/manual/shared.ts");
}

#[test]
fn pattern_parsing() {
    let p = Pattern::try_parse("foo");
    assert_eq!(p.star_index, -1);
    assert!(p.is_valid());

    let p = Pattern::try_parse("foo/*");
    assert_eq!(p.star_index, 4);
    assert!(p.is_valid());
    assert!(p.matches("foo/bar"));
    assert!(!p.matches("baz/bar"));
    assert_eq!(p.matched_text("foo/bar"), "bar");

    let p = Pattern::try_parse("*");
    assert!(p.is_valid());

    let p = Pattern::try_parse("foo*bar*baz");
    assert!(!p.is_valid());
}

#[test]
fn resolve_exports_string_main() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");
    fs.insert_dir("/src/node_modules");
    fs.insert_dir("/src/node_modules/mypkg");
    fs.insert_dir("/src/node_modules/mypkg/dist");
    fs.write_file(
        "/src/node_modules/mypkg/package.json",
        r#"{"name":"mypkg","exports":"./dist/index.js"}"#,
    )
    .unwrap();
    fs.write_file(
        "/src/node_modules/mypkg/dist/index.js",
        "export const x = 1;",
    )
    .unwrap();

    let opts = CompilerOptions::default();
    let state = ResolutionState::new("mypkg", "/src", false, ModuleKind::None, &opts, &fs, "/src");
    let result = state.resolve_node_like();
    assert!(
        result.is_resolved(),
        "expected resolved, got {:?}",
        result.resolved_file_name
    );
    assert_eq!(
        result.resolved_file_name,
        "/src/node_modules/mypkg/dist/index.js"
    );
}

#[test]
fn resolve_exports_conditional_types() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");
    fs.insert_dir("/src/node_modules");
    fs.insert_dir("/src/node_modules/mypkg");
    fs.insert_dir("/src/node_modules/mypkg/dist");
    fs.write_file(
            "/src/node_modules/mypkg/package.json",
            r#"{"name":"mypkg","exports":{".":{"types":"./dist/index.d.ts","default":"./dist/index.js"}}}"#,
        )
        .unwrap();
    fs.write_file(
        "/src/node_modules/mypkg/dist/index.d.ts",
        "export declare const x: number;",
    )
    .unwrap();
    fs.write_file(
        "/src/node_modules/mypkg/dist/index.js",
        "export const x = 1;",
    )
    .unwrap();

    let opts = CompilerOptions::default();
    let state = ResolutionState::new("mypkg", "/src", false, ModuleKind::None, &opts, &fs, "/src");
    let result = state.resolve_node_like();

    assert!(
        result.is_resolved(),
        "expected resolved, got {:?}",
        result.resolved_file_name
    );
    assert_eq!(
        result.resolved_file_name,
        "/src/node_modules/mypkg/dist/index.d.ts"
    );
}

#[test]
fn resolve_exports_subpath() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");
    fs.insert_dir("/src/node_modules");
    fs.insert_dir("/src/node_modules/mypkg");
    fs.insert_dir("/src/node_modules/mypkg/dist");
    fs.write_file(
        "/src/node_modules/mypkg/package.json",
        r#"{"name":"mypkg","exports":{"./feature":"./dist/feature.js"}}"#,
    )
    .unwrap();
    fs.write_file(
        "/src/node_modules/mypkg/dist/feature.js",
        "export const x = 1;",
    )
    .unwrap();

    let opts = CompilerOptions::default();
    let state = ResolutionState::new(
        "mypkg/feature",
        "/src",
        false,
        ModuleKind::None,
        &opts,
        &fs,
        "/src",
    );
    let result = state.resolve_node_like();
    assert!(
        result.is_resolved(),
        "expected resolved, got {:?}",
        result.resolved_file_name
    );
    assert_eq!(
        result.resolved_file_name,
        "/src/node_modules/mypkg/dist/feature.js"
    );
}

#[test]
fn resolve_package_imports_exact() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");
    fs.insert_dir("/src/lib");
    fs.write_file(
        "/src/package.json",
        r##"{"name":"myapp","imports":{"#utils":"./lib/utils.js"}}"##,
    )
    .unwrap();
    fs.write_file("/src/lib/utils.js", "export const x = 1;")
        .unwrap();

    let opts = CompilerOptions::default();
    let state = ResolutionState::new(
        "#utils",
        "/src",
        false,
        ModuleKind::None,
        &opts,
        &fs,
        "/src",
    );
    let result = state.resolve_node_like();
    assert!(
        result.is_resolved(),
        "expected resolved, got {:?}",
        result.resolved_file_name
    );
    assert_eq!(result.resolved_file_name, "/src/lib/utils.js");
}

#[test]
fn resolve_package_imports_pattern() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");
    fs.insert_dir("/src/components");
    fs.write_file(
        "/src/package.json",
        r##"{"name":"myapp","imports":{"#components/*":"./components/*.js"}}"##,
    )
    .unwrap();
    fs.write_file("/src/components/Button.js", "export const Button = 1;")
        .unwrap();

    let opts = CompilerOptions::default();
    let state = ResolutionState::new(
        "#components/Button",
        "/src",
        false,
        ModuleKind::None,
        &opts,
        &fs,
        "/src",
    );
    let result = state.resolve_node_like();
    assert!(
        result.is_resolved(),
        "expected resolved, got {:?}",
        result.resolved_file_name
    );
    assert_eq!(result.resolved_file_name, "/src/components/Button.js");
}

#[test]
fn resolve_package_imports_lone_hash_unresolved() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");
    fs.write_file(
        "/src/package.json",
        r##"{"name":"myapp","imports":{"#a":"./a.js"}}"##,
    )
    .unwrap();

    let opts = CompilerOptions::default();
    let state = ResolutionState::new("#", "/src", false, ModuleKind::None, &opts, &fs, "/src");
    let result = state.resolve_node_like();
    assert!(
        !result.is_resolved(),
        "expected unresolved for lone '#', got {:?}",
        result.resolved_file_name
    );
}

#[test]
fn resolve_package_imports_walks_to_parent_scope() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");
    fs.insert_dir("/src/sub");
    fs.insert_dir("/src/lib");
    fs.write_file(
        "/src/package.json",
        r##"{"name":"myapp","imports":{"#utils":"./lib/utils.js"}}"##,
    )
    .unwrap();
    fs.write_file("/src/lib/utils.js", "export const x = 1;")
        .unwrap();

    let opts = CompilerOptions::default();

    let state = ResolutionState::new(
        "#utils",
        "/src/sub",
        false,
        ModuleKind::None,
        &opts,
        &fs,
        "/src",
    );
    let result = state.resolve_node_like();
    assert!(
        result.is_resolved(),
        "expected resolved, got {:?}",
        result.resolved_file_name
    );
    assert_eq!(result.resolved_file_name, "/src/lib/utils.js");
}

#[test]
fn resolve_types_versions_redirect() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");
    fs.insert_dir("/src/node_modules");
    fs.insert_dir("/src/node_modules/foo");
    fs.insert_dir("/src/node_modules/foo/old");
    fs.insert_dir("/src/node_modules/foo/new");
    fs.write_file(
            "/src/node_modules/foo/package.json",
            r#"{"name":"foo","types":"./old/index.d.ts","typesVersions":{"*":{"*":["./new/index.d.ts"]}}}"#,
        )
        .unwrap();
    fs.write_file("/src/node_modules/foo/old/index.d.ts", "export {}")
        .unwrap();
    fs.write_file("/src/node_modules/foo/new/index.d.ts", "export {}")
        .unwrap();

    let opts = CompilerOptions::default();
    let state = ResolutionState::new("foo", "/src", false, ModuleKind::None, &opts, &fs, "/src");
    let result = state.resolve_node_like();
    assert!(
        result.is_resolved(),
        "expected resolved, got {:?}",
        result.resolved_file_name
    );

    assert_eq!(
        result.resolved_file_name,
        "/src/node_modules/foo/new/index.d.ts"
    );
}

#[test]
fn resolve_types_versions_falls_back_when_no_match() {
    use crate::vfs::InMemoryFS;
    let fs = InMemoryFS::new();
    fs.insert_dir("/src");
    fs.insert_dir("/src/node_modules");
    fs.insert_dir("/src/node_modules/foo");
    fs.write_file(
            "/src/node_modules/foo/package.json",

            r#"{"name":"foo","types":"./index.d.ts","typesVersions":{"*":{"bar":["./other/bar.d.ts"]}}}"#,
        )
        .unwrap();
    fs.write_file("/src/node_modules/foo/index.d.ts", "export {}")
        .unwrap();

    let opts = CompilerOptions::default();
    let state = ResolutionState::new("foo", "/src", false, ModuleKind::None, &opts, &fs, "/src");
    let result = state.resolve_node_like();
    assert!(
        result.is_resolved(),
        "expected resolved, got {:?}",
        result.resolved_file_name
    );
    assert_eq!(
        result.resolved_file_name,
        "/src/node_modules/foo/index.d.ts"
    );
}
