//! Single-file JavaScript and declaration emit ("transpile").
//!
//! Port of Go `internal/transpile` (`TranspileModule`/`TranspileDeclaration`):
//! synthesize a one-file program over an in-memory FS (plus a barebones lib
//! for declaration emit), force the option set single-file transpilation
//! implies, and emit capturing the output text instead of writing to disk.

use std::sync::Arc;

use crate::ast::Diagnostic;
use crate::compiler::{CompilerHostImpl, Program, ProgramOptions};
use crate::core::compiler_options::{CompilerOptions, JsxEmit};
use crate::core::tristate::Tristate;
use crate::tsoptions::ParsedCommandLine;
use crate::tspath;
use crate::vfs::InMemoryFS;

/// Options configuring single-file transpilation (Go `transpile.Options`).
pub struct TranspileOptions {
    /// Base compiler options; several are unconditionally overridden — see
    /// [`transpile_worker`].
    pub compiler_options: CompilerOptions,
    /// Name given to the synthesized input file. The extension controls
    /// parsing (script vs module, JSX allowed, …). Empty → `module.ts`
    /// (or `module.tsx` when `jsx` is set).
    pub file_name: String,
    /// Whether syntactic and compiler-option diagnostics are included in the
    /// result. Diagnostics produced while emitting are always included.
    pub report_diagnostics: bool,
}

/// Result of a transpilation (Go `transpile.Output`).
pub struct TranspileOutput {
    pub output_text: String,
    pub diagnostics: Vec<Diagnostic>,
    pub source_map_text: String,
    /// Message strings from the emit result (this port surfaces emit-time
    /// failures as strings rather than AST diagnostics). Go folds these into
    /// `diagnostics`; the transpile test runner appends them to the
    /// diagnostics baseline section.
    pub emit_notes: Vec<String>,
}

/// Synthetic current directory rooting the single input file (Go
/// `inputDirectory`).
const INPUT_DIRECTORY: &str = "/";

/// Directory the barebones default lib is placed in for declaration
/// transpilation (Go `libDirectory`).
const LIB_DIRECTORY: &str = "/lib";

/// Declaration emit works without a real `lib`, but local inferences need at
/// least a minimal `lib` (the checker types inferred declarations as `any`
/// otherwise; late-bound symbol names need `Symbol`). Verbatim from Go
/// `barebonesLibContent`.
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

/// Transpile a single file of source text to JavaScript (Go
/// `TranspileModule`).
pub fn transpile_module(input: &str, options: TranspileOptions) -> TranspileOutput {
    transpile_worker(input, options, false)
}

/// Create a declaration (.d.ts) file from a single file of source text (Go
/// `TranspileDeclaration`). Because only the single input file is available,
/// the result may differ from what a full program type-check and emit would
/// produce.
pub fn transpile_declaration(input: &str, options: TranspileOptions) -> TranspileOutput {
    transpile_worker(input, options, true)
}

/// Go `transpileWorker`.
pub fn transpile_worker(
    input: &str,
    options: TranspileOptions,
    declaration: bool,
) -> TranspileOutput {
    let mut opts = options.compiler_options.clone();

    // Clear options that do not apply to single-file transpilation.
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

    // Do not set `isolatedModules` if `verbatimModuleSyntax` was supplied,
    // since it would be redundant.
    if !opts.verbatim_module_syntax.is_true() {
        opts.isolated_modules = Tristate::True;
    }
    opts.no_check = Tristate::True;
    opts.no_resolve = Tristate::True;

    // Nothing is written to disk; no input/output path conflict check.
    opts.suppress_output_path_check = Tristate::True;
    // The file name can carry a non-ts extension.
    opts.allow_non_ts_extensions = Tristate::True;

    if declaration {
        opts.declaration = Tristate::True;
        opts.emit_declaration_only = Tristate::True;
        opts.isolated_declarations = Tristate::True;
        // A (barebones) default lib is used for declaration emit.
        opts.no_lib = Tristate::False;
    } else {
        opts.declaration = Tristate::False;
        opts.declaration_map = Tristate::False;
        opts.no_lib = Tristate::True;
    }

    // If jsx is specified, treat the file as .tsx.
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

    // Declaration emit needs a default lib to resolve global types
    // (`Array`, `Symbol`); plain transpilation sets NoLib so none is read.
    // The default lib name depends on the configured target — mirror Go's
    // `tsoptions.GetDefaultLibFileName` by asking the program's lib-name
    // mapping for the (already-cleared) `lib` option.
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
        // Go: syntactic diagnostics of the input source file + config-file
        // parsing diagnostics + program diagnostics. Ours: program
        // construction diagnostics already carry the parse errors of the
        // input files (single-file program → exactly the input's syntactic
        // set).
        for d in program.diagnostics() {
            all_diagnostics.push((**d).clone());
        }
    }

    // `emit` takes a plain `Fn` callback; capture writes via interior mutability.
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

/// Go delegates to `tsoptions.GetDefaultLibFileName`; the program's
/// target → entry-lib mapping (kept in sync with `compiler`).
fn default_lib_file_name(options: &CompilerOptions) -> String {
    let names = crate::compiler::default_lib_file_names(options);
    names.first().cloned().unwrap_or_else(|| "lib.d.ts".to_string())
}
