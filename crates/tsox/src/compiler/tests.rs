use super::*;
use crate::bundled::{BundledFS, lib_path};
use crate::core::compiler_options::CompilerOptions;
use crate::core::tristate::Tristate;
use crate::tsoptions::parse_command_line;
use crate::vfs::{InMemoryFS, OsFS};

#[test]
fn program_parses_input_files() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/a.ts", "let x = 1;");
    fs.insert_file("/proj/b.ts", "let y = ;");

    let args: Vec<String> = vec![
        "--noLib".to_string(),
        "/proj/a.ts".to_string(),
        "/proj/b.ts".to_string(),
    ];
    let parsed = parse_command_line(&args, "/proj", Some(fs.as_ref()));
    let host = Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()));
    let program = Program::new(ProgramOptions {
        config: parsed,
        host,
    });
    assert_eq!(program.source_files().len(), 2);

    assert!(
        program
            .diagnostics()
            .iter()
            .any(|d| d.category == Category::Error)
    );
}

#[test]
fn program_does_not_load_bundled_libs_without_root_files() {
    let fs = Arc::new(BundledFS::new(Arc::new(OsFS)));
    let args: Vec<String> = vec![];
    let parsed = parse_command_line(&args, "/proj", Some(fs.as_ref()));
    let host = Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()));
    let program = Program::new(ProgramOptions {
        config: parsed,
        host,
    });
    assert!(program.source_files().is_empty());
}

#[test]
fn program_loads_bundled_libs_with_root_files() {
    let inner = Arc::new(InMemoryFS::new());
    inner.insert_dir("/proj");
    inner.insert_file("/proj/a.ts", "let x = 1;");
    let fs = Arc::new(BundledFS::new(inner));
    let args: Vec<String> = vec!["/proj/a.ts".to_string()];
    let parsed = parse_command_line(&args, "/proj", Some(fs.as_ref()));
    let host = Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()));
    let program = Program::new(ProgramOptions {
        config: parsed,
        host,
    });

    assert!(program.source_files().len() > 1);
    assert!(
        program
            .source_files()
            .iter()
            .any(|file| file.file_name == "/proj/a.ts")
    );
}

#[test]
fn extract_reference_libs() {
    let text = "/// <reference lib=\"es5\" />\n/// <reference lib=\"dom\" />\ninterface X {}";
    let refs = extract_reference_lib_directives(text);
    assert_eq!(refs, vec!["es5", "dom"]);
}

#[test]
fn program_file_ordering_with_reference_paths() {
    let fs = Arc::new(InMemoryFS::new());

    let files = [
        (
            "/dev/src/index.ts",
            "/// <reference path='/dev/src2/a/5.ts' />\n/// <reference path='/dev/src2/a/10.ts' />",
        ),
        ("/dev/src2/a/5.ts", "/// <reference path='4.ts' />"),
        ("/dev/src2/a/4.ts", "/// <reference path='b/3.ts' />"),
        ("/dev/src2/a/b/3.ts", "/// <reference path='2.ts' />"),
        ("/dev/src2/a/b/2.ts", "/// <reference path='c/1.ts' />"),
        ("/dev/src2/a/b/c/1.ts", "console.log('hello');"),
        ("/dev/src2/a/10.ts", "/// <reference path='b/c/d/9.ts' />"),
        ("/dev/src2/a/b/c/d/9.ts", "/// <reference path='e/8.ts' />"),
        ("/dev/src2/a/b/c/d/e/8.ts", "/// <reference path='7.ts' />"),
        (
            "/dev/src2/a/b/c/d/e/7.ts",
            "/// <reference path='f/6.ts' />",
        ),
        ("/dev/src2/a/b/c/d/e/f/6.ts", "console.log('world!');"),
    ];
    for (name, content) in &files {
        fs.insert_file(name, content);
    }

    let parsed = ParsedCommandLine {
        compiler_options: {
            let mut opts = CompilerOptions::default();
            opts.no_lib = Tristate::True;
            opts
        },
        file_names: vec!["/dev/src/index.ts".to_string()],
        errors: vec![],
        config_file_name: String::new(),
        raw_options: None,
        include: vec![],
        exclude: vec![],
        files_spec: vec![],
        has_include_spec: false,
        has_exclude_spec: false,
        has_files_spec: false,
        references: vec![],
        compile_on_save: None,
        watch: false,
        watch_options: Default::default(),
    };
    let host = Arc::new(CompilerHostImpl::new(
        fs,
        "/dev/src".to_string(),
        lib_path(),
    ));
    let program = Program::new(ProgramOptions {
        config: parsed,
        host,
    });

    let actual: Vec<&str> = program
        .source_files()
        .iter()
        .map(|f| f.file_name.as_str())
        .collect();

    let expected = vec![
        "/dev/src2/a/b/c/1.ts",
        "/dev/src2/a/b/2.ts",
        "/dev/src2/a/b/3.ts",
        "/dev/src2/a/4.ts",
        "/dev/src2/a/5.ts",
        "/dev/src2/a/b/c/d/e/f/6.ts",
        "/dev/src2/a/b/c/d/e/7.ts",
        "/dev/src2/a/b/c/d/e/8.ts",
        "/dev/src2/a/b/c/d/9.ts",
        "/dev/src2/a/10.ts",
        "/dev/src/index.ts",
    ];

    assert_eq!(actual, expected);
}

#[test]
fn program_file_ordering_imports() {
    let fs = Arc::new(InMemoryFS::new());

    for dir in [
        "/dev/src",
        "/dev/src2/a",
        "/dev/src2/a/b",
        "/dev/src2/a/b/c",
        "/dev/src2/a/b/c/d",
        "/dev/src2/a/b/c/d/e",
        "/dev/src2/a/b/c/d/e/f",
    ] {
        fs.insert_dir(dir);
    }
    let files = [
        (
            "/dev/src/index.ts",
            "import * as five from '../src2/a/5.ts';\nimport * as ten from '../src2/a/10.ts';",
        ),
        ("/dev/src2/a/5.ts", "import * as four from './4.ts';"),
        ("/dev/src2/a/4.ts", "import * as three from './b/3.ts';"),
        ("/dev/src2/a/b/3.ts", "import * as two from './2.ts';"),
        ("/dev/src2/a/b/2.ts", "import * as one from './c/1.ts';"),
        ("/dev/src2/a/b/c/1.ts", "console.log('hello');"),
        ("/dev/src2/a/10.ts", "import * as nine from './b/c/d/9.ts';"),
        (
            "/dev/src2/a/b/c/d/9.ts",
            "import * as eight from './e/8.ts';",
        ),
        (
            "/dev/src2/a/b/c/d/e/8.ts",
            "import * as seven from './7.ts';",
        ),
        (
            "/dev/src2/a/b/c/d/e/7.ts",
            "import * as six from './f/6.ts';",
        ),
        ("/dev/src2/a/b/c/d/e/f/6.ts", "console.log('world!');"),
    ];
    for (name, content) in &files {
        fs.insert_file(name, content);
    }

    let parsed = ParsedCommandLine {
        compiler_options: {
            let mut opts = CompilerOptions::default();
            opts.no_lib = Tristate::True;
            opts
        },
        file_names: vec!["/dev/src/index.ts".to_string()],
        ..Default::default()
    };
    let host = Arc::new(CompilerHostImpl::new(
        fs,
        "/dev/src".to_string(),
        lib_path(),
    ));
    let program = Program::new(ProgramOptions {
        config: parsed,
        host,
    });

    let actual: Vec<&str> = program
        .source_files()
        .iter()
        .map(|f| f.file_name.as_str())
        .collect();
    let expected = vec![
        "/dev/src/index.ts",
        "/dev/src2/a/5.ts",
        "/dev/src2/a/10.ts",
        "/dev/src2/a/b/c/d/9.ts",
        "/dev/src2/a/b/c/d/e/8.ts",
        "/dev/src2/a/b/c/d/e/7.ts",
        "/dev/src2/a/b/c/d/e/f/6.ts",
        "/dev/src2/a/4.ts",
        "/dev/src2/a/b/3.ts",
        "/dev/src2/a/b/2.ts",
        "/dev/src2/a/b/c/1.ts",
    ];
    assert_eq!(actual, expected);
}

#[test]
fn program_file_ordering_cycles() {
    let fs = Arc::new(InMemoryFS::new());
    for dir in [
        "/dev/src",
        "/dev/src2/a",
        "/dev/src2/a/b",
        "/dev/src2/a/b/c",
        "/dev/src2/a/b/c/d",
        "/dev/src2/a/b/c/d/e",
        "/dev/src2/a/b/c/d/e/f",
    ] {
        fs.insert_dir(dir);
    }
    let files = [
        (
            "/dev/src/index.ts",
            "import * as five from '../src2/a/5.ts';\nimport * as ten from '../src2/a/10.ts';",
        ),
        ("/dev/src2/a/5.ts", "import * as four from './4.ts';"),
        ("/dev/src2/a/4.ts", "import * as three from './b/3.ts';"),
        (
            "/dev/src2/a/b/3.ts",
            "import * as two from './2.ts';\nimport * as cycle from '/dev/src/index.ts';",
        ),
        ("/dev/src2/a/b/2.ts", "import * as one from './c/1.ts';"),
        ("/dev/src2/a/b/c/1.ts", "console.log('hello');"),
        ("/dev/src2/a/10.ts", "import * as nine from './b/c/d/9.ts';"),
        (
            "/dev/src2/a/b/c/d/9.ts",
            "import * as eight from './e/8.ts';\nimport * as cycle from '/dev/src/index.ts';",
        ),
        (
            "/dev/src2/a/b/c/d/e/8.ts",
            "import * as seven from './7.ts';",
        ),
        (
            "/dev/src2/a/b/c/d/e/7.ts",
            "import * as six from './f/6.ts';",
        ),
        ("/dev/src2/a/b/c/d/e/f/6.ts", "console.log('world!');"),
    ];
    for (name, content) in &files {
        fs.insert_file(name, content);
    }

    let parsed = ParsedCommandLine {
        compiler_options: {
            let mut opts = CompilerOptions::default();
            opts.no_lib = Tristate::True;
            opts
        },
        file_names: vec!["/dev/src/index.ts".to_string()],
        ..Default::default()
    };
    let host = Arc::new(CompilerHostImpl::new(
        fs,
        "/dev/src".to_string(),
        lib_path(),
    ));
    let program = Program::new(ProgramOptions {
        config: parsed,
        host,
    });

    let actual: Vec<&str> = program
        .source_files()
        .iter()
        .map(|f| f.file_name.as_str())
        .collect();
    let expected = vec![
        "/dev/src/index.ts",
        "/dev/src2/a/5.ts",
        "/dev/src2/a/10.ts",
        "/dev/src2/a/b/c/d/9.ts",
        "/dev/src2/a/b/c/d/e/8.ts",
        "/dev/src2/a/b/c/d/e/7.ts",
        "/dev/src2/a/b/c/d/e/f/6.ts",
        "/dev/src2/a/4.ts",
        "/dev/src2/a/b/3.ts",
        "/dev/src2/a/b/2.ts",
        "/dev/src2/a/b/c/1.ts",
    ];
    assert_eq!(actual, expected);
}

#[test]
fn program_resolves_module_imports() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/src");
    fs.insert_file(
        "/src/main.ts",
        "import { foo } from \"./foo\"; export const x = foo;",
    );
    fs.insert_file("/src/foo.ts", "export const foo: number = 42;");

    let parsed = ParsedCommandLine {
        compiler_options: {
            let mut opts = CompilerOptions::default();
            opts.no_lib = Tristate::True;
            opts
        },
        file_names: vec!["/src/main.ts".to_string()],
        ..Default::default()
    };
    let host = Arc::new(CompilerHostImpl::new(
        fs,
        "/src".to_string(),
        "lib.d.ts".to_string(),
    ));
    let program = Program::new(ProgramOptions {
        config: parsed,
        host,
    });

    assert_eq!(program.source_files().len(), 2);
    assert!(
        program.get_source_file("/src/foo.ts").is_some(),
        "expected /src/foo.ts to be loaded via import resolution"
    );
    assert!(
        program.get_source_file("/src/main.ts").is_some(),
        "expected /src/main.ts to be loaded as a root file"
    );
}

#[test]
fn program_resolves_transitive_module_imports() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/src");
    fs.insert_file(
        "/src/a.ts",
        "import { b } from \"./b\"; export const a = b;",
    );
    fs.insert_file(
        "/src/b.ts",
        "import { c } from \"./c\"; export const b = c;",
    );
    fs.insert_file("/src/c.ts", "export const c: number = 3;");

    let parsed = ParsedCommandLine {
        compiler_options: {
            let mut opts = CompilerOptions::default();
            opts.no_lib = Tristate::True;
            opts
        },
        file_names: vec!["/src/a.ts".to_string()],
        ..Default::default()
    };
    let host = Arc::new(CompilerHostImpl::new(
        fs,
        "/src".to_string(),
        "lib.d.ts".to_string(),
    ));
    let program = Program::new(ProgramOptions {
        config: parsed,
        host,
    });

    assert_eq!(program.source_files().len(), 3);
    assert!(program.get_source_file("/src/b.ts").is_some());
    assert!(program.get_source_file("/src/c.ts").is_some());
}

#[test]
fn include_processor_diagnostics_with_missing_file_casing() {
    let fs = Arc::new(InMemoryFS::with_case_sensitivity(true));
    fs.insert_dir("/src");

    fs.insert_file("/src/myFile.ts", "export const y = 2;");

    let parsed = ParsedCommandLine {
        compiler_options: {
            let mut opts = CompilerOptions::default();
            opts.no_lib = Tristate::True;
            opts.skip_lib_check = Tristate::True;
            opts
        },

        file_names: vec!["/src/MyFile.ts".to_string(), "/src/myFile.ts".to_string()],
        errors: vec![],
        config_file_name: String::new(),
        raw_options: None,
        include: vec![],
        exclude: vec![],
        files_spec: vec![],
        has_include_spec: false,
        has_exclude_spec: false,
        has_files_spec: false,
        references: vec![],
        compile_on_save: None,
        watch: false,
        watch_options: Default::default(),
    };
    let host = Arc::new(CompilerHostImpl::new(fs, "/".to_string(), lib_path()));
    let program = Program::new(ProgramOptions {
        config: parsed,
        host,
    });

    let diags = program.diagnostics();
    assert!(
        diags.iter().any(|d| d.category == Category::Error),
        "expected at least one error diagnostic for missing /src/MyFile.ts, got: {:?}",
        diags
    );

    assert!(
        program.get_source_file("/src/myFile.ts").is_some(),
        "expected /src/myFile.ts to be loaded"
    );
}

#[test]
fn extract_reference_path_directives_resolves_relative() {
    let text = "/// <reference path='./b/3.ts' />\n/// <reference path='/abs/4.ts' />";
    let refs = extract_reference_path_directives(text, "/dev/src2/a/5.ts");
    assert_eq!(refs, vec!["/dev/src2/a/b/3.ts", "/abs/4.ts"]);
}

#[test]
fn extract_reference_path_directives_single_quotes() {
    let text = "/// <reference path='b/3.ts' />";
    let refs = extract_reference_path_directives(text, "/dev/src2/a/5.ts");
    assert_eq!(refs, vec!["/dev/src2/a/b/3.ts"]);
}

fn parse_bundled_lib(lib_name: &str) -> Vec<crate::parser::ParserDiagnostic> {
    let content = crate::bundled::lib_contents(lib_name)
        .unwrap_or_else(|| panic!("bundled lib '{lib_name}' not found"));
    let (_file, diags) = crate::parser::Parser::parse_source_file_text_with_diagnostics(
        &format!("/bundled/{lib_name}"),
        content.to_string(),
    );
    diags
}

fn assert_no_parser_errors(lib_name: &str, diags: &[crate::parser::ParserDiagnostic]) {
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.message.category == crate::diagnostics::Category::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "{lib_name} should parse with zero errors, got {}:\n{}",
        errors.len(),
        errors
            .iter()
            .map(|d| format!("  {:?}: {}", d.message.code, d.message.text))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn bundled_lib_es2015_iterable_parses_without_errors() {
    let diags = parse_bundled_lib("lib.es2015.iterable.d.ts");
    assert_no_parser_errors("lib.es2015.iterable.d.ts", &diags);
}

#[test]
fn bundled_lib_dom_parses_without_errors() {
    let diags = parse_bundled_lib("lib.dom.d.ts");
    assert_no_parser_errors("lib.dom.d.ts", &diags);
}

#[test]
fn bundled_lib_es5_parses_without_errors() {
    let diags = parse_bundled_lib("lib.es5.d.ts");
    assert_no_parser_errors("lib.es5.d.ts", &diags);
}

#[test]
fn bundled_lib_es2015_collection_parses_without_errors() {
    let diags = parse_bundled_lib("lib.es2015.collection.d.ts");
    assert_no_parser_errors("lib.es2015.collection.d.ts", &diags);
}

#[test]
fn bundled_lib_decorators_parses_without_errors() {
    let diags = parse_bundled_lib("lib.decorators.d.ts");
    assert_no_parser_errors("lib.decorators.d.ts", &diags);
}

#[test]
fn node_modules_js_skipped_when_allow_js_false() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/src");
    fs.insert_dir("/proj/node_modules");
    fs.insert_dir("/proj/node_modules/mypkg");

    fs.insert_file(
        "/proj/node_modules/mypkg/index.js",
        "module.exports = { x: 1 };\nfunction f(a, b) { return a + b; }\n",
    );
    fs.insert_file(
        "/proj/node_modules/mypkg/package.json",
        r#"{"name": "mypkg", "version": "1.0.0", "main": "index.js"}"#,
    );
    fs.insert_file(
        "/proj/src/main.ts",
        "import * as pkg from 'mypkg';\nexport const v = pkg;",
    );

    let parsed = ParsedCommandLine {
        compiler_options: {
            let mut opts = CompilerOptions::default();
            opts.no_lib = Tristate::True;
            opts
        },
        file_names: vec!["/proj/src/main.ts".to_string()],
        ..Default::default()
    };
    let host = Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()));
    let program = Program::new(ProgramOptions {
        config: parsed,
        host,
    });

    assert!(
        program
            .get_source_file("/proj/node_modules/mypkg/index.js")
            .is_none(),
        "expected node_modules .js file to be skipped when allowJs is false"
    );

    let has_syntax_error = program
        .diagnostics()
        .iter()
        .any(|d| d.code == 1003 || d.code == 1005);
    assert!(
        !has_syntax_error,
        "expected no TS1003/TS1005 syntax diagnostics from node_modules .js, got: {:?}",
        program
            .diagnostics()
            .iter()
            .map(|d| (d.code, d.message_args.clone()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn node_modules_js_loaded_when_allow_js_true() {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_dir("/proj/src");
    fs.insert_dir("/proj/node_modules");
    fs.insert_dir("/proj/node_modules/mypkg");
    fs.insert_file("/proj/node_modules/mypkg/index.js", "export const x = 1;\n");
    fs.insert_file(
        "/proj/node_modules/mypkg/package.json",
        r#"{"name": "mypkg", "version": "1.0.0", "main": "index.js"}"#,
    );
    fs.insert_file(
        "/proj/src/main.ts",
        "import { x } from 'mypkg';\nexport const v = x;",
    );

    let parsed = ParsedCommandLine {
        compiler_options: {
            let mut opts = CompilerOptions::default();
            opts.no_lib = Tristate::True;
            opts.allow_js = Tristate::True;
            opts
        },
        file_names: vec!["/proj/src/main.ts".to_string()],
        ..Default::default()
    };
    let host = Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()));
    let program = Program::new(ProgramOptions {
        config: parsed,
        host,
    });

    assert!(
        program
            .get_source_file("/proj/node_modules/mypkg/index.js")
            .is_some(),
        "expected node_modules .js file to be loaded when allowJs is true; files: {:?}",
        program
            .source_files()
            .iter()
            .map(|f| f.file_name.as_str())
            .collect::<Vec<_>>()
    );
}
