use std::sync::Arc;

use crate::ast::Diagnostic;
use crate::compiler::{CompilerHostImpl, Program, ProgramOptions};
use crate::core::compiler_options::{CompilerOptions, JsxEmit};
use crate::core::tristate::Tristate;
use crate::tsoptions::ParsedCommandLine;
use crate::tspath;
use crate::vfs::InMemoryFS;

pub struct TranspileOptions {
    pub compiler_options: CompilerOptions,

    pub file_name: String,

    pub report_diagnostics: bool,
}

pub struct TranspileOutput {
    pub output_text: String,
    pub diagnostics: Vec<Diagnostic>,
    pub source_map_text: String,

    pub emit_notes: Vec<String>,
}

const INPUT_DIRECTORY: &str = "/";

const LIB_DIRECTORY: &str = "/lib";

const BAREBONES_LIB_CONTENT: &str = "interface Boolean {}
interface Function {}
interface CallableFunction {}
interface NewableFunction {}
interface IArguments {}
interface Number {}
interface Object {}
interface RegExp {}
interface String {}
interface Array<T> { length: number; [n: number]: T; }
interface SymbolConstructor {
    (desc?: string | number): symbol;
    for(name: string): symbol;
    readonly toStringTag: symbol;
}
declare var Symbol: SymbolConstructor;
interface Symbol {
    readonly [Symbol.toStringTag]: string;
}";

pub fn transpile_module(input: &str, options: TranspileOptions) -> TranspileOutput {
    transpile_worker(input, options, false)
}

pub fn transpile_declaration(input: &str, options: TranspileOptions) -> TranspileOutput {
    transpile_worker(input, options, true)
}

pub fn transpile_worker(
    input: &str,
    options: TranspileOptions,
    declaration: bool,
) -> TranspileOutput {
    let mut opts = options.compiler_options.clone();

    opts.incremental = Tristate::Unknown;
    opts.declaration = Tristate::Unknown;
    opts.emit_declaration_only = Tristate::Unknown;
    opts.no_emit = Tristate::Unknown;
    opts.lib = Vec::new();
    opts.out_file = String::new();
    opts.composite = Tristate::Unknown;
    opts.ts_build_info_file = String::new();
    opts.paths = None;
    opts.root_dirs = Vec::new();
    opts.types = Vec::new();
    opts.allow_importing_ts_extensions = Tristate::Unknown;
    opts.no_emit_on_error = Tristate::Unknown;
    opts.declaration_dir = String::new();

    if !opts.verbatim_module_syntax.is_true() {
        opts.isolated_modules = Tristate::True;
    }
    opts.no_check = Tristate::True;
    opts.no_resolve = Tristate::True;

    opts.suppress_output_path_check = Tristate::True;

    opts.allow_non_ts_extensions = Tristate::True;

    if declaration {
        opts.declaration = Tristate::True;
        opts.emit_declaration_only = Tristate::True;
        opts.isolated_declarations = Tristate::True;

        opts.no_lib = Tristate::False;
    } else {
        opts.declaration = Tristate::False;
        opts.declaration_map = Tristate::False;
        opts.no_lib = Tristate::True;
    }

    let mut file_name = options.file_name;
    if file_name.is_empty() {
        file_name = if opts.jsx != JsxEmit::None {
            "module.tsx"
        } else {
            "module.ts"
        }
        .to_string();
    }
    let input_file_name = tspath::get_normalized_absolute_path(&file_name, INPUT_DIRECTORY);

    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/");
    fs.insert_file(&input_file_name, input);

    if declaration {
        let lib_file_name = default_lib_file_name(&opts);
        let lib_path = tspath::combine_paths(LIB_DIRECTORY, &[&lib_file_name]);
        fs.insert_dir(LIB_DIRECTORY);
        fs.insert_file(&lib_path, BAREBONES_LIB_CONTENT);
    }

    let host = Arc::new(CompilerHostImpl::new(
        fs.clone() as Arc<dyn crate::vfs::FS>,
        INPUT_DIRECTORY.to_string(),
        LIB_DIRECTORY.to_string(),
    ));

    let config = ParsedCommandLine {
        compiler_options: opts.clone(),
        file_names: vec![input_file_name.clone()],
        ..Default::default()
    };
    let program = Arc::new(Program::new(ProgramOptions { config, host }));

    let mut all_diagnostics: Vec<Diagnostic> = Vec::new();
    if options.report_diagnostics {
        for d in program.diagnostics() {
            all_diagnostics.push((**d).clone());
        }
    }

    let captured = std::cell::RefCell::new((String::new(), String::new()));
    let emit_result = program.emit(&mut |file_name, text| {
        let mut slots = captured.borrow_mut();
        if file_name.ends_with(".map") {
            slots.1 = text.to_string();
        } else {
            slots.0 = text.to_string();
        }
        std::io::Result::Ok(())
    });
    let (output_text, source_map_text) = captured.into_inner();
    let emit_notes = emit_result.diagnostics;

    TranspileOutput {
        output_text,
        diagnostics: all_diagnostics,
        source_map_text,
        emit_notes,
    }
}

fn default_lib_file_name(options: &CompilerOptions) -> String {
    let names = crate::compiler::default_lib_file_names(options);
    names
        .first()
        .cloned()
        .unwrap_or_else(|| "lib.d.ts".to_string())
}
