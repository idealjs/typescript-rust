#![allow(unused_imports)]
#![allow(dead_code)]

use super::m4v_3::IncludeProcessor;

use std::collections::HashSet;
use std::sync::Arc;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::text::TextRange;
use tsox_core::diagnostics::Message;
use tsox_core::diagnostics::messages_generated as msg;
use tsox_core::tspath::directory_separator::Path;
use tsox_frontend::ast::diagnostic::{Diagnostic, DiagnosticsCollection};
use tsox_frontend::ast::mig::m3d_2::new_compiler_diagnostic;
use tsox_frontend::ast::node_source_file::SourceFile;

use crate::compiler::Program;
use crate::mig::m4u_3::{EmitHostImpl, EmitHost};
use tsox_emit::mig::m3n_5::r33k8_defs::DeclarationEmitHost as BridgeDeclarationEmitHost;
use tsox_emit::mig::m3n_5::r33k8_defs::DeclarationEmitHostFns;
use tsox_checker::binder::referenceresolver::{
    new_reference_resolver, ReferenceResolver, ReferenceResolverHooks,
};
use tsox_core::core::compiler_options_kinds::{ModuleKind, NewLineKind, ScriptTarget};
use tsox_core::core::tristate::Tristate;
use tsox_core::core::mig::m3j::compute_ecma_line_starts;
use tsox_core::core::text::TextPos;
use tsox_core::diagnostics::Category;
use tsox_core::stringutil::encode_uri;
use tsox_core::stringutil::mig::m3m_2::add_utf8_byte_order_mark;
use tsox_core::tspath::mig::m3i::{compare_paths, get_relative_path_from_directory};
use tsox_core::tspath::{
    combine_paths, ensure_trailing_directory_separator, file_extension_is, get_directory_path,
    get_normalized_absolute_path, get_relative_path_to_directory_or_url, get_root_length,
    normalize_path, normalize_slashes, ComparePathsOptions,
};
use tsox_emit::mig::m3m::{TransformOptions, Transformer};
use tsox_emit::mig::r33k6_shim::HasFileName;
use tsox_emit::mig::m3n_5::r33k8_defs::OutputPathsValue as OutputPaths;
use tsox_emit::mig::m3n_6::new_supplemental_references_transformer
as declarations_new_supplemental_references_transformer;
use tsox_emit::mig::m3n_7::new_declaration_transformer as declarations_new_declaration_transformer;
use tsox_emit::mig::m4k_2::new_es_module_transformer as module_transforms_new_es_module_transformer;
use tsox_emit::mig::m4k_2::new_implied_module_transformer
as module_transforms_new_implied_module_transformer;
use tsox_emit::mig::m4k_3::new_import_elision_transformer as ts_transforms_new_import_elision_transformer;
use tsox_emit::mig::wt1::get_es_transformer as es_transforms_get_es_transformer;
use tsox_emit::mig::wt1b::new_legacy_decorators_transformer
as ts_transforms_new_legacy_decorators_transformer;
use tsox_emit::printer::EmitContext;
use tsox_frontend::ast::node_source_file::{FileReference, LanguageVariant, ScriptKind};
use tsox_frontend::ast::utilities::is_in_js_file;
use tsox_frontend::ast::{Node, SyntaxKind};
use tsox_frontend::format::mig::m4o_2::{
    new_printer as printer_new_printer, EmitContext as PrinterEmitContext, EmitTextWriter,
    Generator as SourceMapGenerator, PrintHandlers as PrinterHandlers, Printer, PrinterOptions,
};
use tsox_frontend::scanner::skip_trivia;
use tsox_tsoptions::mig::m5h_5::create_diagnostic_for_node_in_source_file;
use tsox_tsoptions::mig::m5i2::{
    for_each_property_assignment, get_callback_for_finding_property_assignment_by_value,
    get_options_syntax_by_array_element_value,
};
use tsox_tsoptions::mig::m5j::get_ts_config_prop_array_element_value;
use tsox_tsoptions::module::PackageId;

pub const EXTENSION_JSON: &str = ".json";

pub type SourceMapSource = Arc<SourceFile>;

pub static SUPPLEMENTAL_VIRTUAL_FILE_PRODUCED_BY_THE_CONTENT_MAPPER_FOR_FILE_0: Message = Message {
    code: 100055,
    category: Category::Message,
    key: "Supplemental_virtual_file_produced_by_the_content_mapper_for_file_0_100055",
    text: "Supplemental virtual file produced by the content mapper for file '{0}'.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub struct SourceMapEmitResult {
    pub input_source_file_names: Vec<String>,
    pub source_map: String,
    pub generated_file: String,
}

pub struct EmitResult {
    pub emit_skipped: bool,
    pub emitted_files: Vec<String>,
    pub source_maps: Vec<SourceMapEmitResult>,
    pub diagnostics: Vec<String>,
}

#[derive(Default)]
pub struct WriteFileData {
    pub source_map_url_pos: i32,
    pub diagnostics: Vec<Diagnostic>,
    pub source_file: Option<Arc<SourceFile>>,
    pub skipped_dts_write: bool,
}

pub trait SourceFileMayBeEmittedHost: Send + Sync {
    fn options(&self) -> &CompilerOptions;
    fn get_project_reference_from_source(
        &self,
        path: &str,
    ) -> Option<tsox_tsoptions::mig::m5h_3::SourceOutputAndProjectReference>;
    fn is_source_file_from_external_library(&self, file: &Arc<SourceFile>) -> bool;
    fn is_source_file_from_project_reference(&self, file: &Arc<SourceFile>) -> bool;
    fn get_current_directory(&self) -> String;
    fn use_case_sensitive_file_names(&self) -> bool;
    fn source_files(&self) -> Vec<Arc<SourceFile>>;
}

pub fn printer_get_emit_context() -> EmitContext { ::tsox_core::fntrace::enter("printer_get_emit_context"); 
    tsox_emit::printer::mig::m4m_3::get_emit_context().0
}

fn writer_is_at_start_of_line(writer: &EmitTextWriter, new_line: &str) -> bool { ::tsox_core::fntrace::enter("writer_is_at_start_of_line"); 
    // Go textwriter.go:84 lineStart:空文本或以换行结尾(Rust writer 未跟踪 lineStart,按可观察状态等价)
    let text = writer.string();
    text.is_empty() || text.ends_with(new_line)
}

fn source_file_with_node(file: &Arc<SourceFile>, node: Arc<Node>) -> Arc<SourceFile> { ::tsox_core::fntrace::enter("source_file_with_node"); 
    // Go TransformSourceFile 返回的仍是同一 SourceFile 视图;Rust 变换器产出 Arc<Node>,
    // 以原文件元数据重建包装(m4d_4 copy_from 同思路)
    Arc::new(SourceFile {
        node,
        file_name: file.file_name.clone(),
        text: file.text.clone(),
        line_map: tsox_frontend::ast::node::LineMap {
            line_starts: file.line_map.line_starts.clone(),
        },
        language_variant: file.language_variant,
        script_kind: file.script_kind,
        comment_directives: file.comment_directives.clone(),
        jsdoc_cache: std::sync::RwLock::new(std::collections::HashMap::new()),
        has_lazy_jsdoc: file.has_lazy_jsdoc,
        is_declaration_file: file.is_declaration_file,
        imports: file.imports.clone(),
        module_augmentations: file.module_augmentations.clone(),
        ambient_module_names: file.ambient_module_names.clone(),
        parse_error_spans: file.parse_error_spans.clone(),
        external_module_indicator: file.external_module_indicator.clone(),
        common_js_module_indicator: file.common_js_module_indicator.clone(),
        uses_uri_style_node_core_modules: file.uses_uri_style_node_core_modules,
        has_parse_diagnostics: file.has_parse_diagnostics,
        referenced_files: file.referenced_files.clone(),
        type_reference_directives: file.type_reference_directives.clone(),
        lib_reference_directives: file.lib_reference_directives.clone(),
        supplemental_source_files: file.supplemental_source_files.clone(),
    })
}

fn empty_print_handlers() -> PrinterHandlers { ::tsox_core::fntrace::enter("empty_print_handlers"); 
    PrinterHandlers {
        has_global_name: None,
        map_source_position: None,
        on_before_emit_node: None,
        on_after_emit_node: None,
        on_before_emit_node_list: None,
        on_after_emit_node_list: None,
        on_before_emit_token: None,
        on_after_emit_token: None,
    }
}

fn script_target_display(target: ScriptTarget) -> String { ::tsox_core::fntrace::enter("script_target_display"); 
    match target {
        ScriptTarget::None => String::new(),
        ScriptTarget::ES5 => "ES5".to_string(),
        ScriptTarget::ES2015 => "ES2015".to_string(),
        ScriptTarget::ES2016 => "ES2016".to_string(),
        ScriptTarget::ES2017 => "ES2017".to_string(),
        ScriptTarget::ES2018 => "ES2018".to_string(),
        ScriptTarget::ES2019 => "ES2019".to_string(),
        ScriptTarget::ES2020 => "ES2020".to_string(),
        ScriptTarget::ES2021 => "ES2021".to_string(),
        ScriptTarget::ES2022 => "ES2022".to_string(),
        ScriptTarget::ES2023 => "ES2023".to_string(),
        ScriptTarget::ES2024 => "ES2024".to_string(),
        ScriptTarget::ES2025 => "ES2025".to_string(),
        ScriptTarget::ESNext => "ESNext".to_string(),
        ScriptTarget::JSON => "JSON".to_string(),
    }
}

pub fn printer_put_emit_context(_context: EmitContext) { ::tsox_core::fntrace::enter("printer_put_emit_context"); }

pub fn sourcemap_new_generator(
    generated_file_name: &str,
    source_root: &str,
    source_map_directory: &str,
    options: &ComparePathsOptions,
) -> SourceMapGenerator { ::tsox_core::fntrace::enter("sourcemap_new_generator"); 
    SourceMapGenerator
}

pub fn get_base_filename(path: &str) -> String { ::tsox_core::fntrace::enter("get_base_filename"); 
    tsox_core::tspath::get_base_file_name(path)
}

pub fn get_source_file_path_in_new_dir(
    file_name: &str,
    new_dir_path: &str,
    current_directory: &str,
    common_source_directory: &str,
    use_case_sensitive_file_names: bool,
) -> String { ::tsox_core::fntrace::enter("get_source_file_path_in_new_dir"); 
    get_source_file_path_in_new_dir_worker(
        file_name,
        new_dir_path,
        current_directory,
        common_source_directory,
        use_case_sensitive_file_names,
    )
}

pub fn get_source_file_path_in_new_dir_worker(
    file_name: &str,
    new_dir_path: &str,
    current_directory: &str,
    common_source_directory: &str,
    use_case_sensitive_file_names: bool,
) -> String { ::tsox_core::fntrace::enter("get_source_file_path_in_new_dir_worker");
    let source_file_path = tsox_core::tspath::get_normalized_absolute_path(file_name, current_directory);
    let source_file_path = match tsox_core::tspath::mig::m3j::trim_file_path_prefix(
        &source_file_path,
        common_source_directory,
        use_case_sensitive_file_names,
    ) {
        (trimmed, true) => trimmed,
        (path, false) => path,
    };
    combine_paths(new_dir_path, &[&source_file_path])
}

pub fn get_common_source_directory(
    options: &CompilerOptions,
    files: impl Fn() -> Vec<String>,
    current_directory: &str,
    use_case_sensitive_file_names: bool,
) -> String { ::tsox_core::fntrace::enter("get_common_source_directory"); 
    let mut common_source_directory;
    if !options.root_dir.is_empty() {
        common_source_directory = options.root_dir.clone();
    } else if !options.config_file_path.is_empty() {
        common_source_directory = get_directory_path(&options.config_file_path);
    } else {
        common_source_directory =
            compute_common_source_directory_of_filenames(&files(), current_directory, use_case_sensitive_file_names);
    }
    if !common_source_directory.is_empty() {
        common_source_directory = ensure_trailing_directory_separator(&common_source_directory);
    }
    common_source_directory
}

fn compute_common_source_directory_of_filenames(
    file_names: &[String],
    current_directory: &str,
    use_case_sensitive_file_names: bool,
) -> String { ::tsox_core::fntrace::enter("compute_common_source_directory_of_filenames"); 
    let mut common_path_components: Vec<String> = Vec::new();
    let mut have_common = false;
    for source_file in file_names {
        let mut source_path_components =
            tsox_core::tspath::mig::m3i::get_normalized_path_components(source_file, current_directory);
        source_path_components.pop();
        if !have_common {
            common_path_components = source_path_components;
            have_common = true;
            continue;
        }
        let n = common_path_components.len().min(source_path_components.len());
        let mut mismatch_at = None;
        for i in 0..n {
            if tsox_core::tspath::get_canonical_file_name(&common_path_components[i], use_case_sensitive_file_names)
                != tsox_core::tspath::get_canonical_file_name(&source_path_components[i], use_case_sensitive_file_names)
            {
                mismatch_at = Some(i);
                break;
            }
        }
        if let Some(i) = mismatch_at {
            if i == 0 {
                return String::new();
            }
            common_path_components.truncate(i);
        }
        if source_path_components.len() < common_path_components.len() {
            common_path_components.truncate(source_path_components.len());
        }
    }
    if common_path_components.is_empty() {
        return current_directory.to_string();
    }
    let mut path = common_path_components.join("/");
    path.push('/');
    path
}

pub fn ts_transforms_new_metadata_transformer(opts: &TransformOptions) -> Arc<Transformer> { ::tsox_core::fntrace::enter("ts_transforms_new_metadata_transformer"); 
    tsox_emit::mig::m4l_2::MetadataTransformer::new_metadata_transformer(opts)
}

pub fn ts_transforms_new_type_eraser_transformer(opts: &TransformOptions) -> Arc<Transformer> { ::tsox_core::fntrace::enter("ts_transforms_new_type_eraser_transformer"); 
    tsox_emit::mig::m4l_6::TypeEraserTransformer::new_type_eraser_transformer(opts)
}

pub fn ts_transforms_new_runtime_syntax_transformer(opts: &TransformOptions) -> Transformer { ::tsox_core::fntrace::enter("ts_transforms_new_runtime_syntax_transformer"); 
    Transformer::new(runtime_syntax_transformer_visit, Some(opts.context.clone()))
}

fn runtime_syntax_transformer_visit(_tx: &mut Transformer, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("runtime_syntax_transformer_visit"); 
    Some(node)
}

pub fn is_source_file_js(file: &SourceFile) -> bool { ::tsox_core::fntrace::enter("is_source_file_js"); 
    file.script_kind == ScriptKind::Js || file.script_kind == ScriptKind::Jsx
}

pub fn is_json_source_file(file: &SourceFile) -> bool { ::tsox_core::fntrace::enter("is_json_source_file"); 
    file.script_kind == ScriptKind::Json
}

pub fn node_is_synthesized(node: &Node) -> bool { ::tsox_core::fntrace::enter("node_is_synthesized"); 
    (node.loc.pos() as i32) < 0 || (node.end() as i32) < 0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmitOnly {
    #[default]
    EmitAll,
    EmitOnlyJs,
    EmitOnlyDts,
    EmitOnlyBuilderSignature,
}

pub type WriteFileFn = Box<dyn Fn(&str, &str, Option<&WriteFileData>) -> Result<(), String> + Send + Sync>;

pub struct Emitter {
    pub host: Arc<EmitHostImpl>,
    pub emit_only: EmitOnly,
    pub emitter_diagnostics: DiagnosticsCollection,
    pub writer: EmitTextWriter,
    pub paths: Option<OutputPaths>,
    pub source_file: Option<Arc<SourceFile>>,
    pub emit_result: EmitResult,
    pub force_emit: bool,
    pub write_file: Option<WriteFileFn>,
}

impl Emitter {
    pub fn run_script_transformers(
        &mut self,
        emit_context: &EmitContext,
        mut source_file: Arc<SourceFile>,
    ) -> Arc<SourceFile> { ::tsox_core::fntrace::enter("run_script_transformers"); 
        for transformer in get_script_transformers(emit_context, self.host.clone(), &source_file) {
            // Go: transformer.TransformSourceFile(sourceFile) = visitor.VisitSourceFile。
            // Rust 变换器以 Arc<Node> 驱动,输出节点重建 SourceFile 包装。
            let mut transformer =
                Arc::into_inner(transformer).expect("script transformer arc uniquely owned");
            let node = transformer
                .transform_source_file(Arc::clone(&source_file.node))
                .unwrap_or_else(|| Arc::clone(&source_file.node));
            source_file = source_file_with_node(&source_file, node);
        }
        source_file
    }

    pub fn run_declaration_transformers(
        &mut self,
        emit_context: &EmitContext,
        mut source_file: Arc<SourceFile>,
        declaration_file_path: &str,
        declaration_map_path: &str,
    ) -> (Arc<SourceFile>, Vec<Diagnostic>) { ::tsox_core::fntrace::enter("run_declaration_transformers"); 
        let mut diags = Vec::new();
        let force_dts_emit = self.emit_only == EmitOnly::EmitOnlyBuilderSignature
            || (self.force_emit && self.emit_only == EmitOnly::EmitOnlyDts);
        let bridge = new_declaration_emit_host_bridge(&self.host);

        // Go: tstransforms.NewDeclarationTransformer(...).TransformSourceFile(sourceFile)
        let mut declaration_transformer = declarations_new_declaration_transformer(
            bridge.clone(),
            emit_context,
            self.host.options().clone(),
            declaration_file_path.to_string(),
            declaration_map_path.to_string(),
        );
        let node = declaration_transformer
            .transform_source_file_entry(Arc::clone(&source_file))
            .unwrap_or_else(|| Arc::clone(&source_file.node));
        source_file = source_file_with_node(&source_file, node);
        diags.extend(declaration_transformer.get_diagnostics());

        // Go: declarations.NewSupplementalReferencesTransformer(...).TransformSourceFile(sourceFile)
        // m3n_6 以 &mut SourceFile 就地追加 referenced_files,对独立副本做变更
        let mut supplemental = declarations_new_supplemental_references_transformer(
            bridge,
            &source_file,
            declaration_file_path.to_string(),
            force_dts_emit,
        );
        let mut copy = source_file_with_node(&source_file, Arc::clone(&source_file.node));
        if let Some(copy_ref) = Arc::get_mut(&mut copy) {
            supplemental.transform_source_file(copy_ref);
        }
        source_file = copy;
        diags.extend(supplemental.get_diagnostics());

        (source_file, diags)
    }

    pub fn emit_js_file(&mut self, source_file: Option<Arc<SourceFile>>, js_file_path: &str, source_map_file_path: &str) { ::tsox_core::fntrace::enter("emit_js_file"); 
        let options = self.host.options().clone();
        let Some(source_file) = source_file else { return };
        if self.emit_only != EmitOnly::EmitAll && self.emit_only != EmitOnly::EmitOnlyJs || js_file_path.is_empty() {
            return;
        }
        if !self.force_emit && (options.no_emit.is_true() || self.host.is_emit_blocked(js_file_path)) {
            self.emit_result.emit_skipped = true;
            return;
        }
        let emit_context = printer_get_emit_context();
        let source_file = self.run_script_transformers(&emit_context, source_file);
        let printer_options = PrinterOptions {
            remove_comments: options.remove_comments.is_true(),
            new_line: options.new_line,
            omit_trailing_semicolon: false,
            no_emit_helpers: options.no_emit_helpers.is_true(),
            target: options.target,
            source_map: options.source_map.is_true(),
            inline_source_map: options.inline_source_map.is_true(),
            inline_sources: options.inline_sources.is_true(),
            omit_brace_source_map_positions: false,
            only_print_jsdoc_style: false,
            never_ascii_escape: false,
            preserve_source_newlines: false,
            terminate_unterminated_literals: false,
        };
        let mut printer = printer_new_printer(printer_options, empty_print_handlers(), PrinterEmitContext);
        let emit_maps = should_emit_source_maps(&options, &source_file);
        self.print_source_file(js_file_path, source_map_file_path, source_file, &mut printer, &options, emit_maps);
        printer_put_emit_context(emit_context);
    }

    pub fn emit_declaration_file(&mut self, source_file: Option<Arc<SourceFile>>, declaration_file_path: &str, declaration_map_path: &str) { ::tsox_core::fntrace::enter("emit_declaration_file"); 
        let options = self.host.options().clone();
        let Some(source_file) = source_file else { return };
        if self.emit_only == EmitOnly::EmitOnlyJs || declaration_file_path.is_empty() {
            return;
        }
        let emit_declaration_map = self.emit_only != EmitOnly::EmitOnlyBuilderSignature
            && options.declaration_map.is_true();
        let content_mapped_source = source_file.clone();

        let emit_context = printer_get_emit_context();
        let (source_file, diags) =
            self.run_declaration_transformers(&emit_context, source_file, declaration_file_path, declaration_map_path);
        for elem in diags.iter() {
            self.emitter_diagnostics.add(elem.clone());
        }

        if !self.force_emit
            && self.emit_only != EmitOnly::EmitOnlyBuilderSignature
            && (options.no_emit.is_true() || self.host.is_emit_blocked(declaration_file_path))
        {
            self.emit_result.emit_skipped = true;
            return;
        }
        let decl_blocked = !diags.is_empty()
            && !self.force_emit
            && self.emit_only != EmitOnly::EmitOnlyBuilderSignature;
        if decl_blocked {
            self.emit_result.emit_skipped = true;
            return;
        }

        let printer_options = PrinterOptions {
            remove_comments: options.remove_comments.is_true(),
            new_line: options.new_line,
            omit_trailing_semicolon: false,
            no_emit_helpers: true,
            target: options.get_emit_script_target(),
            source_map: emit_declaration_map,
            inline_source_map: options.inline_source_map.is_true(),
            inline_sources: false,
            omit_brace_source_map_positions: true,
            only_print_jsdoc_style: true,
            never_ascii_escape: false,
            preserve_source_newlines: false,
            terminate_unterminated_literals: false,
        };

        // Go: printHandlers.MapSourcePosition 按 contentMappedSource.SpanMap() 安装声明映射闭包。
        // SourceFile 尚无 span_map 字段、PrintHandlers.map_source_position 为 fn 指针无法捕获环境，
        // 待 tsox-frontend 扩展后回装（见 progress_notes_r51k01.md 交接 3）。
        let _ = (&content_mapped_source, emit_declaration_map);
        let print_handlers = empty_print_handlers();
        let mut printer = printer_new_printer(printer_options, print_handlers, PrinterEmitContext);

        let declaration_map_options = CompilerOptions {
            source_map: if emit_declaration_map { Tristate::True } else { Tristate::False },
            source_root: options.source_root.clone(),
            map_root: options.map_root.clone(),
            ..Default::default()
        };
        let emit_maps = should_emit_source_maps(&declaration_map_options, &source_file);
        self.print_source_file(declaration_file_path, declaration_map_path, source_file, &mut printer, &declaration_map_options, emit_maps);
        printer_put_emit_context(emit_context);
    }

    pub fn print_source_file(
        &mut self,
        js_file_path: &str,
        source_map_file_path: &str,
        source_file: Arc<SourceFile>,
        printer_: &mut Printer,
        map_options: &CompilerOptions,
        should_emit_source_maps_flag: bool,
    ) { ::tsox_core::fntrace::enter("print_source_file"); 
        let options = self.host.options();
        let emit_bom = options.emit_bom.is_true();
        let mut source_map_generator = None;
        if should_emit_source_maps_flag {
            source_map_generator = Some(sourcemap_new_generator(
                &get_base_filename(&normalize_slashes(js_file_path)),
                &get_source_root(map_options),
                &self.get_source_map_directory(map_options, js_file_path, Some(&source_file)),
                &ComparePathsOptions {
                    use_case_sensitive_file_names: self.host.use_case_sensitive_file_names(),
                    current_directory: self.host.get_current_directory(),
                },
            ));
        }

        printer_.write(
            &source_file.node,
            Some(&source_file),
            self.writer.clone(),
            source_map_generator.as_mut(),
        );

        let mut source_map_url_pos = -1;
        if let Some(generator) = source_map_generator.as_ref() {
            if map_options.source_map.is_true() || map_options.inline_source_map.is_true() {
                // Go: generator.Sources() / generator.RawSourceMap()。
                // m4o_2::Generator 为桩(printer 侧 source map 记录管线未移植),先以空值占位,
                // 待 tsox-frontend Generator 落地后回装(交接 1)。
                let _ = generator;
                self.emit_result.source_maps.push(SourceMapEmitResult {
                    input_source_file_names: Vec::new(),
                    source_map: String::new(),
                    generated_file: js_file_path.to_string(),
                });
            }
            let source_mapping_url = self.get_source_mapping_url(
                map_options,
                generator,
                js_file_path,
                source_map_file_path,
                Some(&source_file),
            );
            if !source_mapping_url.is_empty() {
                let new_line_character = options.new_line.get_new_line_character();
                if !writer_is_at_start_of_line(&self.writer, new_line_character) {
                    let nl = if options.new_line == NewLineKind::CRLF { "\r\n" } else { "\n" };
                    self.writer.write(nl);
                }
                source_map_url_pos = self.writer.string().len() as i32;
                self.writer.write("//# sourceMappingURL=");
                self.writer.write(&source_mapping_url);
            }
            if !source_map_file_path.is_empty() {
                // Go: generator.ToJSON()。同上,m4o_2::Generator 桩,占位空串(交接 1)。
                let source_map = String::new();
                let data = WriteFileData {
                    source_file: self.source_file.clone(),
                    ..Default::default()
                };
                match self.write_text(source_map_file_path, &source_map, &data) {
                    Err(err) => {
                        let diag = new_compiler_diagnostic(
                            msg::COULD_NOT_WRITE_FILE_0_COLON_1,
                            vec![js_file_path.to_string(), err],
                        );
                        self.emitter_diagnostics.add(diag);
                    }
                    Ok(()) => {
                        self.emit_result.emitted_files.push(source_map_file_path.to_string());
                    }
                }
            }
        } else {
            self.writer.write_line("");
        }

        let mut text = self.writer.string();
        if emit_bom {
            text = add_utf8_byte_order_mark(&text);
        }
        let data = WriteFileData {
            source_map_url_pos,
            diagnostics: self.emitter_diagnostics.get_all(),
            source_file: self.source_file.clone(),
            ..Default::default()
        };
        let skipped_dts_write = data.skipped_dts_write;
        match self.write_text(js_file_path, &text, &data) {
            Err(err) => {
                let diag = new_compiler_diagnostic(
                    msg::COULD_NOT_WRITE_FILE_0_COLON_1,
                    vec![js_file_path.to_string(), err],
                );
                self.emitter_diagnostics.add(diag);
            }
            Ok(()) => {
                if !skipped_dts_write {
                    self.emit_result.emitted_files.push(js_file_path.to_string());
                }
            }
        }
        self.writer.clear();
    }

    pub fn write_text(&mut self, file_name: &str, text: &str, data: &WriteFileData) -> Result<(), String> { ::tsox_core::fntrace::enter("write_text"); 
        if let Some(write_file) = self.write_file.as_ref() {
            return write_file(file_name, text, Some(data));
        }
        self.host.write_file(file_name, text).map_err(|err| err.to_string())
    }

    pub fn get_source_map_directory(&self, map_options: &CompilerOptions, file_path: &str, source_file: Option<&Arc<SourceFile>>) -> String { ::tsox_core::fntrace::enter("get_source_map_directory"); 
        if !map_options.source_root.is_empty() {
            return self.host.common_source_directory();
        }
        if !map_options.map_root.is_empty() {
            let mut source_map_dir = normalize_slashes(&map_options.map_root);
            if let Some(source_file) = source_file {
                source_map_dir = get_directory_path(&get_source_file_path_in_new_dir(
                    &source_file.file_name,
                    &source_map_dir,
                    &self.host.get_current_directory(),
                    &self.host.common_source_directory(),
                    self.host.use_case_sensitive_file_names(),
                ));
            }
            if get_root_length(&source_map_dir) == 0 {
                source_map_dir = combine_paths(&self.host.common_source_directory(), &[&source_map_dir]);
            }
            return source_map_dir;
        }
        get_directory_path(&normalize_path(file_path))
    }

    pub fn get_source_mapping_url(
        &self,
        map_options: &CompilerOptions,
        source_map_generator: &SourceMapGenerator,
        file_path: &str,
        source_map_file_path: &str,
        source_file: Option<&Arc<SourceFile>>,
    ) -> String { ::tsox_core::fntrace::enter("get_source_mapping_url"); 
        if map_options.inline_source_map.is_true() {
            // Go: generator.ToBase64DataURL()。m4o_2::Generator 桩,占位空串(交接 1)。
            let _ = source_map_generator;
            return String::new();
        }
        let source_map_file = get_base_filename(&normalize_slashes(source_map_file_path));
        let map_root = map_options.map_root.clone();
        if !map_root.is_empty() {
            let mut source_map_dir = normalize_slashes(&map_root);
            if let Some(source_file) = source_file {
                source_map_dir = get_directory_path(&get_source_file_path_in_new_dir(
                    &source_file.file_name,
                    &source_map_dir,
                    &self.host.get_current_directory(),
                    &self.host.common_source_directory(),
                    self.host.use_case_sensitive_file_names(),
                ));
            }
            if get_root_length(&source_map_dir) == 0 {
                source_map_dir = combine_paths(&self.host.common_source_directory(), &[&source_map_dir]);
                return encode_uri(&get_relative_path_to_directory_or_url(
                    &get_directory_path(&normalize_path(file_path)),
                    &combine_paths(&source_map_dir, &[&source_map_file]),
                    /* is_absolute_path_an_url */ true,
                    &ComparePathsOptions {
                        use_case_sensitive_file_names: self.host.use_case_sensitive_file_names(),
                        current_directory: self.host.get_current_directory(),
                    },
                ));
            }
            return encode_uri(&combine_paths(&source_map_dir, &[&source_map_file]));
        }
        encode_uri(&source_map_file)
    }
}

#[derive(Clone)]
pub struct DeclarationMapSource {
    file_name: String,
    text: String,
    line_map: Vec<TextPos>,
}

pub fn new_declaration_map_source(source_file: &Arc<SourceFile>) -> DeclarationMapSource { ::tsox_core::fntrace::enter("new_declaration_map_source"); 
    // Go: OriginalText()/OriginalFileName()。Rust SourceFile 尚无重定向/内容映射字段，
    // 无重定向时 original 即当前值（交接 6）。
    let text = source_file.text.clone();
    DeclarationMapSource {
        file_name: source_file.file_name.clone(),
        text,
        line_map: compute_ecma_line_starts(&source_file.text),
    }
}

impl DeclarationMapSource {
    pub fn file_name(&self) -> &str { ::tsox_core::fntrace::enter("file_name"); 
        &self.file_name
    }
    pub fn text(&self) -> &str { ::tsox_core::fntrace::enter("text"); 
        &self.text
    }
    pub fn ecma_line_map(&self) -> &[TextPos] { ::tsox_core::fntrace::enter("ecma_line_map"); 
        &self.line_map
    }
}

pub fn get_module_transformer(opts: &TransformOptions) -> Transformer { ::tsox_core::fntrace::enter("get_module_transformer"); 
    match opts.compiler_options.get_emit_module_kind() {
        ModuleKind::Preserve => module_transforms_new_es_module_transformer(opts),
        ModuleKind::ESNext
        | ModuleKind::ES2022
        | ModuleKind::ES2020
        | ModuleKind::ES2015
        | ModuleKind::Node20
        | ModuleKind::Node18
        | ModuleKind::Node16
        | ModuleKind::NodeNext
        | ModuleKind::CommonJS => module_transforms_new_implied_module_transformer(opts),
        // Go: default → NewCommonJSModuleTransformer(opts)。
        // m4j_2 的 4 参构造返回 CommonJsModuleTransformer，与 Transformer 尚无桥接
        // （m4k_2 implied 路径同样 unimplemented!），先走 implied 保持 format 驱动，
        // 桥接补齐后改回直接构造（见 progress_notes_r51k01.md 交接 5）。
        _ => module_transforms_new_implied_module_transformer(opts),
    }
}

fn jsx_transforms_new_jsx_transformer(opts: &TransformOptions) -> Transformer { ::tsox_core::fntrace::enter("jsx_transforms_new_jsx_transformer"); 
    // Go: jsxtransforms.NewJSXTransformer(opts) 返回 *Transformer。
    // m4i_13::JSXTransformer 状态结构尚无 visit_entry/Transformer 桥接,
    // 按 wt1 shell 模式构造状态后返回身份 visit(交接 5)。
    let _tx = tsox_emit::mig::m4i_13::new_jsx_transformer(opts);
    Transformer::new(runtime_syntax_transformer_visit, Some(opts.context.clone()))
}

fn es_transforms_new_use_strict_transformer(opts: &TransformOptions) -> Transformer { ::tsox_core::fntrace::enter("es_transforms_new_use_strict_transformer"); 
    // Go: estransforms.NewUseStrictTransformer(opts)。m4i::UseStrictTransformer 状态结构
    // 尚无 visit_entry/Transformer 桥接,同 shell 模式(交接 5)。
    let _tx = tsox_emit::mig::m4i::new_use_strict_transformer(opts);
    Transformer::new(runtime_syntax_transformer_visit, Some(opts.context.clone()))
}

fn inliners_new_const_enum_inlining_transformer(opts: &TransformOptions) -> Transformer { ::tsox_core::fntrace::enter("inliners_new_const_enum_inlining_transformer"); 
    // Go: inliners.NewConstEnumInliningTransformer(opts)。m4i_12 状态结构同上(交接 5)。
    let _tx = tsox_emit::mig::m4i_12::new_const_enum_inlining_transformer(opts);
    Transformer::new(runtime_syntax_transformer_visit, Some(opts.context.clone()))
}

pub fn get_script_transformers(
    emit_context: &EmitContext,
    host: Arc<EmitHostImpl>,
    source_file: &Arc<SourceFile>,
) -> Vec<Arc<Transformer>> { ::tsox_core::fntrace::enter("get_script_transformers"); 
    let mut tx: Vec<Arc<Transformer>> = Vec::new();
    let options = host.options();

    let import_elision_enabled = !options.verbatim_module_syntax.is_true() && !is_in_js_file(&source_file.node);
    let jsx_transform_enabled =
        options.get_jsx_transform_enabled() && source_file.language_variant == LanguageVariant::Jsx;

    let emit_resolver = host.get_emit_resolver();

    let reference_resolver: Arc<dyn ReferenceResolver> = if import_elision_enabled
        || jsx_transform_enabled
        || !options.get_isolated_modules()
        || options.emit_decorator_metadata.is_true()
    {
        // Go: referenceResolver = emitResolver(EmitResolver 实现 binder.ReferenceResolver,
        // 各方法委托 getReferenceResolver,见 tsox-checker m2d)。
        emit_resolver.clone()
    } else {
        Arc::new(new_reference_resolver(
            Some(Arc::new(options.clone())),
            ReferenceResolverHooks::default(),
        ))
    };

    let format_host = host.clone();
    let opts = TransformOptions {
        context: emit_context,
        compiler_options: options,
        resolver: reference_resolver,
        emit_resolver: (*emit_resolver).clone(),
        get_emit_module_format_of_file: Arc::new(move |file: &dyn HasFileName| {
            <Program as tsox_checker::checker::Program>::get_emit_module_format_of_file(
                &format_host.program,
                file.file_name(),
            )
        }),
    };

    if options.emit_decorator_metadata.is_true() {
        tx.push(ts_transforms_new_metadata_transformer(&opts));
    }
    tx.push(ts_transforms_new_type_eraser_transformer(&opts));
    if import_elision_enabled {
        tx.push(Arc::new(ts_transforms_new_import_elision_transformer(&opts)));
    }
    tx.push(Arc::new(ts_transforms_new_runtime_syntax_transformer(&opts)));
    if options.experimental_decorators.is_true() {
        if let Some(legacy) = ts_transforms_new_legacy_decorators_transformer(&opts) {
            tx.push(Arc::new(*legacy));
        }
    }
    if jsx_transform_enabled {
        tx.push(Arc::new(jsx_transforms_new_jsx_transformer(&opts)));
    }
    if let Some(downleveler) = es_transforms_get_es_transformer(&opts) {
        tx.push(Arc::new(*downleveler));
    }
    tx.push(Arc::new(es_transforms_new_use_strict_transformer(&opts)));
    tx.push(Arc::new(get_module_transformer(&opts)));
    if !options.get_isolated_modules() {
        tx.push(Arc::new(inliners_new_const_enum_inlining_transformer(&opts)));
    }
    tx
}

pub fn should_emit_source_maps(map_options: &CompilerOptions, source_file: &Arc<SourceFile>) -> bool { ::tsox_core::fntrace::enter("should_emit_source_maps"); 
    (map_options.source_map.is_true() || map_options.inline_source_map.is_true())
        && !file_extension_is(&source_file.file_name, EXTENSION_JSON)
}

pub fn get_source_root(map_options: &CompilerOptions) -> String { ::tsox_core::fntrace::enter("get_source_root"); 
    let mut source_root = normalize_slashes(&map_options.source_root);
    if !source_root.is_empty() {
        source_root = ensure_trailing_directory_separator(&source_root);
    }
    source_root
}

pub fn source_file_may_be_emitted(
    source_file: &Arc<SourceFile>,
    host: &dyn SourceFileMayBeEmittedHost,
    force_dts_emit: bool,
    force_js_emit: bool,
) -> bool { ::tsox_core::fntrace::enter("source_file_may_be_emitted"); 
    let options = host.options();
    if !force_js_emit && options.no_emit_for_js_files.is_true() && is_source_file_js(source_file) {
        return false;
    }
    if source_file.is_declaration_file {
        return false;
    }
    // Go: sourceFile.ContentMapper() != nil 时才返回 false。Rust SourceFile 尚无
    // content_mapper 字段（无内容映射建模），当前恒等价于 ContentMapper() == nil（交接 6）。
    if host.is_source_file_from_external_library(source_file) {
        return false;
    }
    if force_dts_emit || force_js_emit {
        return true;
    }
    // Path 键在本迁移中以 file_name 承载（全库 SourceFile 无独立 path 字段）
    if host.get_project_reference_from_source(&source_file.file_name).is_some() {
        return false;
    }
    if !is_json_source_file(source_file) {
        return true;
    }
    if options.out_dir.is_empty() {
        return false;
    }
    if !options.root_dir.is_empty() || !options.config_file_path.is_empty()
    {
        let common_dir = get_normalized_absolute_path(
            &get_common_source_directory(
                options,
                || Vec::new(),
                &host.get_current_directory(),
                host.use_case_sensitive_file_names(),
            ),
            &host.get_current_directory(),
        );
        let output_path = get_source_file_path_in_new_dir_worker(
            &source_file.file_name,
            &options.out_dir,
            &host.get_current_directory(),
            &common_dir,
            host.use_case_sensitive_file_names(),
        );
        if compare_paths(
            &source_file.file_name,
            &output_path,
            &ComparePathsOptions {
                use_case_sensitive_file_names: host.use_case_sensitive_file_names(),
                current_directory: host.get_current_directory(),
            },
        ) == 0
        {
            return false;
        }
    }
    true
}

pub fn get_source_files_to_emit(
    host: &dyn SourceFileMayBeEmittedHost,
    target_source_files: Option<Vec<Arc<SourceFile>>>,
    force_dts_emit: bool,
    force_js_emit: bool,
) -> Vec<Arc<SourceFile>> { ::tsox_core::fntrace::enter("get_source_files_to_emit"); 
    let target_source_files = target_source_files.unwrap_or_else(|| host.source_files());
    target_source_files
        .into_iter()
        .filter(|source_file| source_file_may_be_emitted(source_file, host, force_dts_emit, force_js_emit))
        .collect()
}

pub fn is_source_file_not_json(file: &Arc<SourceFile>) -> bool { ::tsox_core::fntrace::enter("is_source_file_not_json"); 
    !is_json_source_file(file)
}

pub fn new_declaration_emit_host_bridge(host: &Arc<EmitHostImpl>) -> BridgeDeclarationEmitHost { ::tsox_core::fntrace::enter("new_declaration_emit_host_bridge"); 
    let get_source_file_from_reference_host = host.clone();
    let get_output_paths_for_host = host.clone();
    let source_file_may_be_emitted_host = host.clone();
    let effective_flags_host = host.clone();
    let emit_resolver_host = host.clone();
    BridgeDeclarationEmitHost::new(DeclarationEmitHostFns {
        get_current_directory: Arc::new({
            let host = host.clone();
            move || host.get_current_directory()
        }),
        use_case_sensitive_file_names: Arc::new({
            let host = host.clone();
            move || host.use_case_sensitive_file_names()
        }),
        get_source_file_from_reference: Arc::new(
            move |origin: &Arc<SourceFile>, reference: &tsox_emit::mig::m3n_5::r33k8_defs::FileReference| {
                get_source_file_from_reference_host.get_source_file_from_reference(
                    origin,
                    &tsox_frontend::ast::node_source_file::FileReference {
                        range: reference.text_range,
                        file_name: reference.file_name.clone(),
                        resolution_mode: reference.resolution_mode.unwrap_or_default(),
                        preserve: reference.preserve,
                    },
                )
            },
        ),
        get_output_paths_for: Arc::new(move |file: &SourceFile, force_dts_paths: bool| {
            let file_arc = get_output_paths_for_host
                .program
                .get_source_file_by_path(&file.file_name)
                .expect("output paths: source file missing from program");
            get_output_paths_for_host.get_output_paths_for(&file_arc, force_dts_paths)
        }),
        source_file_may_be_emitted: Arc::new(move |file: &SourceFile, force_dts_emit: bool| {
            let file_arc = source_file_may_be_emitted_host
                .program
                .get_source_file_by_path(&file.file_name)
                .expect("emit check: source file missing from program");
            source_file_may_be_emitted_host
                .source_file_may_be_emitted(&file_arc, force_dts_emit)
        }),
        get_effective_declaration_flags: Arc::new(
            move |node: &Arc<Node>, flags: tsox_frontend::ast::ModifierFlags| {
                effective_flags_host.get_effective_declaration_flags(node, flags)
            },
        ),
        get_emit_resolver: Arc::new(move || (*emit_resolver_host.get_emit_resolver()).clone()),
    })
}

pub fn get_declaration_diagnostics(host: Arc<EmitHostImpl>, file: &Arc<SourceFile>) -> Vec<Diagnostic> { ::tsox_core::fntrace::enter("get_declaration_diagnostics"); 
    let full_files: Vec<Arc<SourceFile>> = get_source_files_to_emit(
        host.as_ref(),
        Some(vec![file.clone()]),
        false,
        false,
    )
    .into_iter()
    .filter(is_source_file_not_json)
    .collect();
    if !full_files.iter().any(|f| Arc::ptr_eq(f, file)) {
        return Vec::new();
    }
    let options = host.options().clone();
    let emit_context = printer_get_emit_context();
    let mut transform = declarations_new_declaration_transformer(
        new_declaration_emit_host_bridge(&host),
        &emit_context,
        options,
        String::new(),
        String::new(),
    );
    // Go: transform.TransformSourceFile(file)(emitter.go:574,结果丢弃仅收诊断)
    transform.transformer.transform_source_file(Arc::clone(&file.node));
    transform.get_diagnostics()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileIncludeKind {
    Import = 0,
    ReferenceFile = 1,
    TypeReferenceDirective = 2,
    LibReferenceDirective = 3,
    RootFile = 4,
    LibFile = 5,
    AutomaticTypeDirectiveFile = 6,
    ContentMapperSupplemental = 7,
}

impl FileIncludeReasonData {
    pub fn clone_data(&self) -> FileIncludeReasonData { ::tsox_core::fntrace::enter("clone_data"); 
        match self {
            FileIncludeReasonData::None => FileIncludeReasonData::None,
            FileIncludeReasonData::Index(index) => FileIncludeReasonData::Index(*index),
            FileIncludeReasonData::ReferencedFile(data) => FileIncludeReasonData::ReferencedFile(Box::new(
                ReferencedFileData { file: data.file.clone(), index: data.index, synthetic: data.synthetic.clone() },
            )),
            FileIncludeReasonData::AutomaticTypeDirectiveFile(data) => {
                FileIncludeReasonData::AutomaticTypeDirectiveFile(Box::new(AutomaticTypeDirectiveFileData {
                    type_reference: data.type_reference.clone(),
                    package_id: data.package_id.clone(),
                }))
            }
            FileIncludeReasonData::CanonicalPath(path) => FileIncludeReasonData::CanonicalPath(path.clone()),
        }
    }
}

#[derive(Clone)]
pub struct ReferencedFileData {
    pub file: Path,
    pub index: usize,
    pub synthetic: Option<Arc<Node>>,
}

#[derive(Clone)]
pub struct AutomaticTypeDirectiveFileData {
    pub type_reference: String,
    pub package_id: Option<PackageId>,
}

#[derive(Clone)]
pub enum FileIncludeReasonData {
    None,
    Index(usize),
    ReferencedFile(Box<ReferencedFileData>),
    AutomaticTypeDirectiveFile(Box<AutomaticTypeDirectiveFileData>),
    CanonicalPath(Path),
}

#[derive(Clone)]
pub struct FileIncludeReason {
    pub kind: FileIncludeKind,
    pub data: FileIncludeReasonData,
    pub relative_file_name_diag: Option<Diagnostic>,
    pub diag: Option<Diagnostic>,
}

impl FileIncludeReason {
    pub fn new(kind: FileIncludeKind, data: FileIncludeReasonData) -> Self { ::tsox_core::fntrace::enter("new"); 
        Self { kind, data, relative_file_name_diag: None, diag: None }
    }

    pub fn is_referenced_file(&self) -> bool { ::tsox_core::fntrace::enter("is_referenced_file"); 
        self.kind <= FileIncludeKind::LibReferenceDirective
    }

    pub fn clone_reason(&self) -> Box<FileIncludeReason> { ::tsox_core::fntrace::enter("clone_reason"); 
        Box::new(FileIncludeReason {
            kind: self.kind,
            data: self.data.clone_data(),
            relative_file_name_diag: None,
            diag: None,
        })
    }

    pub fn as_index(&self) -> usize { ::tsox_core::fntrace::enter("as_index"); 
        match &self.data {
            FileIncludeReasonData::Index(index) => *index,
            _ => panic!("FileIncludeReason data is not an index"),
        }
    }

    pub fn as_lib_file_index(&self) -> Option<usize> { ::tsox_core::fntrace::enter("as_lib_file_index"); 
        match &self.data {
            FileIncludeReasonData::Index(index) => Some(*index),
            _ => None,
        }
    }

    pub fn as_referenced_file_data(&self) -> &ReferencedFileData { ::tsox_core::fntrace::enter("as_referenced_file_data"); 
        match &self.data {
            FileIncludeReasonData::ReferencedFile(data) => data,
            _ => panic!("FileIncludeReason data is not referencedFileData"),
        }
    }

    pub fn as_automatic_type_directive_file_data(&self) -> &AutomaticTypeDirectiveFileData { ::tsox_core::fntrace::enter("as_automatic_type_directive_file_data"); 
        match &self.data {
            FileIncludeReasonData::AutomaticTypeDirectiveFile(data) => data,
            _ => panic!("FileIncludeReason data is not automaticTypeDirectiveFileData"),
        }
    }

    pub fn get_referenced_location(&self, program: &Program) -> ReferenceFileLocation { ::tsox_core::fntrace::enter("get_referenced_location"); 
        let ref_ = self.as_referenced_file_data();
        let file = program
            .get_source_file_by_path(ref_.file.as_str())
            .expect("referenced file missing from program");
        match self.kind {
            FileIncludeKind::Import => {
                let mut specifier = None;
                let mut is_synthetic = false;
                if let Some(synthetic) = ref_.synthetic.clone() {
                    specifier = Some(synthetic);
                    is_synthetic = true;
                } else if ref_.index < file.imports.len() {
                    specifier = Some(file.imports[ref_.index].clone());
                } else {
                    let mut aug_index = file.imports.len();
                    for imp in &file.module_augmentations {
                        if imp.kind == SyntaxKind::StringLiteral {
                            if aug_index == ref_.index {
                                specifier = Some(imp.clone());
                                break;
                            }
                            aug_index += 1;
                        }
                    }
                }
                let resolution = specifier
                    .as_ref()
                    .and_then(|specifier| program.get_resolved_module_from_module_specifier(&file, specifier));
                ReferenceFileLocation {
                    file,
                    node: specifier,
                    ref_: None,
                    package_id: resolution.as_ref().and_then(|r| r.package_id.clone()),
                    is_synthetic,
                }
            }
            FileIncludeKind::ReferenceFile => ReferenceFileLocation {
                file: file.clone(),
                node: None,
                ref_: Some(file.referenced_files[ref_.index].clone()),
                package_id: None,
                is_synthetic: false,
            },
            FileIncludeKind::TypeReferenceDirective => ReferenceFileLocation {
                file: file.clone(),
                node: None,
                ref_: Some(file.type_reference_directives[ref_.index].clone()),
                package_id: None,
                is_synthetic: false,
            },
            FileIncludeKind::LibReferenceDirective => ReferenceFileLocation {
                file: file.clone(),
                node: None,
                ref_: Some(file.lib_reference_directives[ref_.index].clone()),
                package_id: None,
                is_synthetic: false,
            },
            kind => panic!("unknown reason: {:?}", kind),
        }
    }

    pub fn to_diagnostic(self: &Arc<Self>, program: &Program, relative_file_name: bool) -> Option<Diagnostic> { ::tsox_core::fntrace::enter("to_diagnostic"); 
        if relative_file_name {
            self.compute_diagnostic(program, &|file_name: &str| {
                get_relative_path_from_directory(
                    program.get_current_directory(),
                    file_name,
                    &program.compare_paths_options,
                )
            })
        } else {
            self.compute_diagnostic(program, &|file_name: &str| file_name.to_string())
        }
    }

    fn compute_diagnostic(self: &Arc<Self>, program: &Program, to_file_name: &dyn Fn(&str) -> String) -> Option<Diagnostic> { ::tsox_core::fntrace::enter("compute_diagnostic"); 
        if self.is_referenced_file() {
            return self.compute_reference_file_diagnostic(program, to_file_name);
        }
        match self.kind {
            FileIncludeKind::RootFile => {
                if program.opts.config.config_file.is_some() {
                    let config = &program.opts.config;
                    let file_name = get_normalized_absolute_path(
                        &config.file_names[self.as_index()],
                        &program.get_current_directory(),
                    );
                    let matched_file_spec = config.get_matched_file_spec(&file_name);
                    if matched_file_spec.as_deref().is_some_and(|spec| !spec.is_empty()) {
                        return Some(new_compiler_diagnostic(
                            msg::PART_OF_FILES_LIST_IN_TSCONFIG_JSON,
                            vec![matched_file_spec.unwrap_or_default(), to_file_name(&file_name)],
                        ));
                    }
                    let (matched_include_spec, is_default_include_spec) =
                        config.get_matched_include_spec(&file_name).unwrap_or_default();
                    if !matched_include_spec.is_empty() {
                        if is_default_include_spec {
                            return Some(new_compiler_diagnostic(
                                msg::MATCHED_BY_DEFAULT_INCLUDE_PATTERN_ASTERISK_ASTERISK_SLASH_ASTERISK,
                                Vec::new(),
                            ));
                        }
                        return Some(new_compiler_diagnostic(
                            msg::MATCHED_BY_INCLUDE_PATTERN_0_IN_1,
                            vec![matched_include_spec, to_file_name(&config.config_name())],
                        ));
                    }
                    return Some(new_compiler_diagnostic(msg::ROOT_FILE_SPECIFIED_FOR_COMPILATION, Vec::new()));
                }
                Some(new_compiler_diagnostic(msg::ROOT_FILE_SPECIFIED_FOR_COMPILATION, Vec::new()))
            }
            FileIncludeKind::AutomaticTypeDirectiveFile => {
                let data = self.as_automatic_type_directive_file_data();
                if !program.options().uses_wildcard_types() {
                    if let Some(package_id) = &data.package_id {
                        if !package_id.name.is_empty() {
                            return Some(new_compiler_diagnostic(
                                msg::ENTRY_POINT_OF_TYPE_LIBRARY_0_SPECIFIED_IN_COMPILEROPTIONS_WITH_PACKAGEID_1,
                                vec![data.type_reference.clone(), package_id.to_string()],
                            ));
                        }
                    }
                    Some(new_compiler_diagnostic(
                        msg::ENTRY_POINT_OF_TYPE_LIBRARY_0_SPECIFIED_IN_COMPILEROPTIONS,
                        vec![data.type_reference.clone()],
                    ))
                } else if let Some(package_id) = &data.package_id {
                    if !package_id.name.is_empty() {
                        return Some(new_compiler_diagnostic(
                            msg::ENTRY_POINT_FOR_IMPLICIT_TYPE_LIBRARY_0_WITH_PACKAGEID_1,
                            vec![data.type_reference.clone(), package_id.to_string()],
                        ));
                    }
                    Some(new_compiler_diagnostic(
                        msg::ENTRY_POINT_FOR_IMPLICIT_TYPE_LIBRARY_0,
                        vec![data.type_reference.clone()],
                    ))
                } else {
                    Some(new_compiler_diagnostic(
                        msg::ENTRY_POINT_FOR_IMPLICIT_TYPE_LIBRARY_0,
                        vec![data.type_reference.clone()],
                    ))
                }
            }
            FileIncludeKind::LibFile => {
                if let Some(index) = self.as_lib_file_index() {
                    return Some(new_compiler_diagnostic(
                        msg::LIBRARY_0_SPECIFIED_IN_COMPILEROPTIONS,
                        vec![program.options().lib[index].clone()],
                    ));
                }
                let target = script_target_display(program.options().get_emit_script_target());
                if !target.is_empty() {
                    return Some(new_compiler_diagnostic(msg::DEFAULT_LIBRARY_FOR_TARGET_0, vec![target]));
                }
                Some(new_compiler_diagnostic(msg::DEFAULT_LIBRARY, Vec::new()))
            }
            FileIncludeKind::ContentMapperSupplemental => {
                let canonical_path = match &self.data {
                    FileIncludeReasonData::CanonicalPath(path) => path.clone(),
                    _ => panic!("FileIncludeReason data is not a path"),
                };
                let canonical = program
                    .get_source_file_by_path(canonical_path.as_str())
                    .expect("supplemental canonical file missing from program");
                Some(new_compiler_diagnostic(
                    SUPPLEMENTAL_VIRTUAL_FILE_PRODUCED_BY_THE_CONTENT_MAPPER_FOR_FILE_0,
                    vec![to_file_name(&canonical.file_name)],
                ))
            }
            kind => panic!("unknown reason: {:?}", kind),
        }
    }

    fn compute_reference_file_diagnostic(self: &Arc<Self>, program: &Program, to_file_name: &dyn Fn(&str) -> String) -> Option<Diagnostic> { ::tsox_core::fntrace::enter("compute_reference_file_diagnostic"); 
        let reference_location = ip_of(program).get_reference_location(self, program);
        let reference_text = reference_location.text();
        match self.kind {
            FileIncludeKind::Import => {
                let file_name = to_file_name(&reference_location.file.file_name);
                if !reference_location.is_synthetic {
                    if let Some(package_id) = &reference_location.package_id {
                        if !package_id.name.is_empty() {
                            return Some(new_compiler_diagnostic(
                                msg::IMPORTED_VIA_0_FROM_FILE_1_WITH_PACKAGEID_2,
                                vec![reference_text, file_name, package_id.to_string()],
                            ));
                        }
                    }
                    Some(new_compiler_diagnostic(msg::IMPORTED_VIA_0_FROM_FILE_1, vec![reference_text, file_name]))
                } else if program
                    .import_helpers_import_specifiers
                    .get(reference_location.file.file_name.as_str())
                    .map(|specifier| {
                        reference_location
                            .node
                            .as_ref()
                            .map(|node| Arc::ptr_eq(specifier, node))
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
                {
                    if let Some(package_id) = &reference_location.package_id {
                        if !package_id.name.is_empty() {
                            return Some(new_compiler_diagnostic(
                                msg::IMPORTED_VIA_0_FROM_FILE_1_WITH_PACKAGEID_2_TO_IMPORT_IMPORTHELPERS_AS_SPECIFIED_IN_COMPILEROPTIONS,
                                vec![reference_text, file_name, package_id.to_string()],
                            ));
                        }
                    }
                    Some(new_compiler_diagnostic(
                        msg::IMPORTED_VIA_0_FROM_FILE_1_TO_IMPORT_IMPORTHELPERS_AS_SPECIFIED_IN_COMPILEROPTIONS,
                        vec![reference_text, file_name],
                    ))
                } else if let Some(package_id) = &reference_location.package_id {
                    if !package_id.name.is_empty() {
                        return Some(new_compiler_diagnostic(
                            msg::IMPORTED_VIA_0_FROM_FILE_1_WITH_PACKAGEID_2_TO_IMPORT_JSX_AND_JSXS_FACTORY_FUNCTIONS,
                            vec![reference_text, file_name, package_id.to_string()],
                        ));
                    }
                    Some(new_compiler_diagnostic(
                        msg::IMPORTED_VIA_0_FROM_FILE_1_TO_IMPORT_JSX_AND_JSXS_FACTORY_FUNCTIONS,
                        vec![reference_text, file_name],
                    ))
                } else {
                    Some(new_compiler_diagnostic(
                        msg::IMPORTED_VIA_0_FROM_FILE_1_TO_IMPORT_JSX_AND_JSXS_FACTORY_FUNCTIONS,
                        vec![reference_text, file_name],
                    ))
                }
            }
            FileIncludeKind::ReferenceFile => Some(new_compiler_diagnostic(
                msg::REFERENCED_VIA_0_FROM_FILE_1,
                vec![reference_text, to_file_name(&reference_location.file.file_name)],
            )),
            FileIncludeKind::TypeReferenceDirective => {
                if let Some(package_id) = &reference_location.package_id {
                    if !package_id.name.is_empty() {
                        return Some(new_compiler_diagnostic(
                            msg::TYPE_LIBRARY_REFERENCED_VIA_0_FROM_FILE_1_WITH_PACKAGEID_2,
                            vec![reference_text, to_file_name(&reference_location.file.file_name), package_id.to_string()],
                        ));
                    }
                }
                Some(new_compiler_diagnostic(
                    msg::TYPE_LIBRARY_REFERENCED_VIA_0_FROM_FILE_1,
                    vec![reference_text, to_file_name(&reference_location.file.file_name)],
                ))
            }
            FileIncludeKind::LibReferenceDirective => Some(new_compiler_diagnostic(
                msg::LIBRARY_REFERENCED_VIA_0_FROM_FILE_1,
                vec![reference_text, to_file_name(&reference_location.file.file_name)],
            )),
            kind => panic!("unknown reason: {:?}", kind),
        }
    }

    pub fn to_related_info(self: &Arc<Self>, program: &Program) -> Option<Diagnostic> { ::tsox_core::fntrace::enter("to_related_info"); 
        if self.is_referenced_file() {
            return self.compute_reference_file_related_info(program);
        }
        let config = &program.opts.config;
        let config_file = config.config_file.as_ref()?;
        match self.kind {
            FileIncludeKind::RootFile => {
                let file_name = get_normalized_absolute_path(
                    &config.file_names[self.as_index()],
                    &program.get_current_directory(),
                );
                let matched_file_spec = config.get_matched_file_spec(&file_name);
                if matched_file_spec.as_deref().is_some_and(|spec| !spec.is_empty()) {
                    let files_node = get_ts_config_prop_array_element_value(
                        Some(&config_file.source_file),
                        "files",
                        matched_file_spec.as_deref().unwrap_or_default(),
                    );
                    if let Some(files_node) = files_node {
                        return Some(create_diagnostic_for_node_in_source_file(
                            &config_file.source_file,
                            &files_node,
                            msg::FILE_IS_MATCHED_BY_FILES_LIST_SPECIFIED_HERE,
                            Vec::new(),
                        ));
                    }
                } else {
                    let (matched_include_spec, is_default_include_spec) =
                        config.get_matched_include_spec(&file_name).unwrap_or_default();
                    if !matched_include_spec.is_empty() && !is_default_include_spec {
                        let include_node = get_ts_config_prop_array_element_value(
                            Some(&config_file.source_file),
                            "include",
                            &matched_include_spec,
                        );
                        if let Some(include_node) = include_node {
                            return Some(create_diagnostic_for_node_in_source_file(
                                &config_file.source_file,
                                &include_node,
                                msg::FILE_IS_MATCHED_BY_INCLUDE_PATTERN_SPECIFIED_HERE,
                                Vec::new(),
                            ));
                        }
                    }
                }
                None
            }
            FileIncludeKind::AutomaticTypeDirectiveFile => {
                if !program.options().uses_wildcard_types() {
                    let data = self.as_automatic_type_directive_file_data();
                    let types_syntax = get_options_syntax_by_array_element_value(
                        ip_of(program).get_compiler_options_object_literal_syntax(program)
                            .as_deref(),
                        "types",
                        &data.type_reference,
                    );
                    if let Some(types_syntax) = types_syntax {
                        return Some(create_diagnostic_for_node_in_source_file(
                            &config_file.source_file,
                            &types_syntax,
                            msg::FILE_IS_ENTRY_POINT_OF_TYPE_LIBRARY_SPECIFIED_HERE,
                            Vec::new(),
                        ));
                    }
                }
                None
            }
            FileIncludeKind::LibFile => {
                if let Some(index) = self.as_lib_file_index() {
                    let lib_syntax = get_options_syntax_by_array_element_value(
                        ip_of(program).get_compiler_options_object_literal_syntax(program)
                            .as_deref(),
                        "lib",
                        &program.options().lib[index],
                    );
                    if let Some(lib_syntax) = lib_syntax {
                        return Some(create_diagnostic_for_node_in_source_file(
                            &config_file.source_file,
                            &lib_syntax,
                            msg::FILE_IS_LIBRARY_SPECIFIED_HERE,
                            Vec::new(),
                        ));
                    }
                } else {
                    let target = script_target_display(program.options().get_emit_script_target());
                    if !target.is_empty() {
                        let mut callback = get_callback_for_finding_property_assignment_by_value(&target);
                        let target_value_syntax = for_each_property_assignment(
                            ip_of(program)
                                .get_compiler_options_object_literal_syntax(program)
                                .as_deref(),
                            "target",
                            None,
                            &mut callback,
                        );
                        if let Some(target_value_syntax) = target_value_syntax {
                            return Some(create_diagnostic_for_node_in_source_file(
                                &config_file.source_file,
                                &target_value_syntax,
                                msg::FILE_IS_DEFAULT_LIBRARY_FOR_TARGET_SPECIFIED_HERE,
                                Vec::new(),
                            ));
                        }
                    }
                }
                None
            }
            FileIncludeKind::ContentMapperSupplemental => None,
            kind => panic!("unknown reason: {:?}", kind),
        }
    }

    fn compute_reference_file_related_info(self: &Arc<Self>, program: &Program) -> Option<Diagnostic> { ::tsox_core::fntrace::enter("compute_reference_file_related_info"); 
        let reference_location = ip_of(program).get_reference_location(self, program);
        if reference_location.is_synthetic {
            return None;
        }
        match self.kind {
            FileIncludeKind::Import => {
                reference_location.diagnostic_at(msg::FILE_IS_INCLUDED_VIA_IMPORT_HERE, Vec::new())
            }
            FileIncludeKind::ReferenceFile => {
                reference_location.diagnostic_at(msg::FILE_IS_INCLUDED_VIA_REFERENCE_HERE, Vec::new())
            }
            FileIncludeKind::TypeReferenceDirective => {
                reference_location.diagnostic_at(msg::FILE_IS_INCLUDED_VIA_TYPE_LIBRARY_REFERENCE_HERE, Vec::new())
            }
            FileIncludeKind::LibReferenceDirective => {
                reference_location.diagnostic_at(msg::FILE_IS_INCLUDED_VIA_LIBRARY_REFERENCE_HERE, Vec::new())
            }
            kind => panic!("unknown reason: {:?}", kind),
        }
    }
}

pub struct ReferenceFileLocation {
    pub file: Arc<SourceFile>,
    pub node: Option<Arc<Node>>,
    pub ref_: Option<FileReference>,
    pub package_id: Option<PackageId>,
    pub is_synthetic: bool,
}

impl ReferenceFileLocation {
    pub fn text(&self) -> String { ::tsox_core::fntrace::enter("text"); 
        if let Some(node) = &self.node {
            if !node_is_synthesized(node) {
                let text = &self.file.text;
                let start = skip_trivia(text, node.loc.pos()) as usize;
                return text[start..node.end() as usize].to_string();
            }
            return format!("\"{}\"", node.text());
        }
        let ref_ = self.ref_.as_ref().unwrap();
        self.file.text[ref_.range.pos() as usize..ref_.range.end() as usize].to_string()
    }

    pub fn diagnostic_at(&self, message: Message, args: Vec<String>) -> Option<Diagnostic> { ::tsox_core::fntrace::enter("diagnostic_at"); 
        if let Some(node) = &self.node {
            Some(create_diagnostic_for_node_in_source_file(
                &self.file, node, message, args,
            ))
        } else {
            let ref_ = self.ref_.as_ref().unwrap();
            Some(Diagnostic::new(
                Some(self.file.clone()),
                ref_.range,
                message,
                args,
            ))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessingDiagnosticKind {
    UnknownReference,
    ExplainingFileInclude,
}

pub enum ProcessingDiagnosticData {
    FileIncludeReason(Box<FileIncludeReason>),
    IncludeExplainingDiagnostic(Box<IncludeExplainingDiagnostic>),
}

pub struct ProcessingDiagnostic {
    pub kind: ProcessingDiagnosticKind,
    pub data: ProcessingDiagnosticData,
}

impl ProcessingDiagnostic {
    pub fn as_file_include_reason(&self) -> &FileIncludeReason { ::tsox_core::fntrace::enter("as_file_include_reason"); 
        match &self.data {
            ProcessingDiagnosticData::FileIncludeReason(reason) => reason,
            _ => panic!("processingDiagnostic data is not a FileIncludeReason"),
        }
    }

    pub fn as_include_explaining_diagnostic(&self) -> &IncludeExplainingDiagnostic { ::tsox_core::fntrace::enter("as_include_explaining_diagnostic"); 
        match &self.data {
            ProcessingDiagnosticData::IncludeExplainingDiagnostic(diag) => diag,
            _ => panic!("processingDiagnostic data is not an includeExplainingDiagnostic"),
        }
    }

    pub fn create_diagnostic_explaining_file(&self, program: &Program) -> Option<Diagnostic> { ::tsox_core::fntrace::enter("create_diagnostic_explaining_file"); 
        let diag = self.as_include_explaining_diagnostic();
        let mut include_details: Vec<Diagnostic> = Vec::new();
        let mut related_info: Vec<Diagnostic> = Vec::new();
        let mut redirect_info: Vec<Diagnostic> = Vec::new();
        let mut preferred_location: Option<Arc<FileIncludeReason>> = None;
        let mut seen_reasons: HashSet<*const FileIncludeReason> = HashSet::new();
        // Go: diagnosticReason 为 *FileIncludeReason（可能为 nil，isReferencedFile 判空安全）。
        let diagnostic_reason = if diag.diagnostic_reason.is_null() {
            None
        } else {
            Some(unsafe { &*diag.diagnostic_reason })
        };
        if let Some(reason) = diagnostic_reason {
            if reason.is_referenced_file() {
                let reason_arc = find_reason_arc(program, diag.diagnostic_reason);
                if reason_arc
                    .as_ref()
                    .map(|r| !ip_of(program).get_reference_location(r, program).is_synthetic)
                    .unwrap_or(false)
                {
                    preferred_location = reason_arc;
                }
            }
        }

        let mut process_related_info = |include_reason: &Arc<FileIncludeReason>,
                                        preferred_location: &mut Option<Arc<FileIncludeReason>>,
                                        related_info: &mut Vec<Diagnostic>| {
            let reason_preferred = preferred_location
                .as_ref()
                .map(|p| Arc::ptr_eq(p, include_reason))
                .unwrap_or(false);
            if preferred_location.is_none()
                && include_reason.is_referenced_file()
                && !ip_of(program).get_reference_location(include_reason, program).is_synthetic
            {
                *preferred_location = Some(include_reason.clone());
            } else if !reason_preferred {
                if let Some(info) = ip_of(program).get_related_info(include_reason, program) {
                    related_info.push(info);
                }
            }
        };
        let mut process_include = |include_reason: &Arc<FileIncludeReason>,
                                   preferred_location: &mut Option<Arc<FileIncludeReason>>,
                                   include_details: &mut Vec<Diagnostic>,
                                   related_info: &mut Vec<Diagnostic>,
                                   seen_reasons: &mut HashSet<*const FileIncludeReason>| {
            if !seen_reasons.insert(Arc::as_ptr(include_reason)) {
                return;
            }
            include_details.push(include_reason.to_diagnostic(program, false).unwrap());
            process_related_info(include_reason, preferred_location, related_info);
        };

        if !diag.file.0.is_empty() {
            let reasons = ip_of(program).file_include_reasons
                .get(&diag.file)
                .cloned()
                .unwrap_or_default();
            include_details = Vec::with_capacity(reasons.len());
            for reason in &reasons {
                process_include(reason, &mut preferred_location, &mut include_details, &mut related_info, &mut seen_reasons);
            }
            redirect_info = ip_of(program).explain_redirect_and_implied_format(
                program,
                &diag.file,
                &|file_name: &str| file_name.to_string(),
            );
        }
        if let Some(diagnostic_reason) = &diag.diagnostic_reason_opt {
            let reason = Arc::new((**diagnostic_reason).clone());
            process_include(&reason, &mut preferred_location, &mut include_details, &mut related_info, &mut seen_reasons);
        }

        let mut chain: Vec<Diagnostic> = Vec::new();
        if !include_details.is_empty() && (preferred_location.is_none() || seen_reasons.len() != 1) {
            let mut file_reason = new_compiler_diagnostic(msg::THE_FILE_IS_IN_THE_PROGRAM_BECAUSE_COLON, Vec::new());
            file_reason.message_chain = include_details.clone();
            chain.push(file_reason);
        }
        chain.extend(redirect_info);

        let mut result = preferred_location
            .and_then(|location| {
                ip_of(program).get_reference_location(&location, program)
                    .diagnostic_at(diag.message.clone(), diag.args.clone())
            })
            .unwrap_or_else(|| new_compiler_diagnostic(diag.message.clone(), diag.args.clone()));
        if !chain.is_empty() {
            result.message_chain = chain;
        }
        if !related_info.is_empty() {
            result.related_information = related_info;
        }
        Some(result)
    }
}

fn find_reason_arc(program: &Program, reason: *const FileIncludeReason) -> Option<Arc<FileIncludeReason>> { ::tsox_core::fntrace::enter("find_reason_arc"); 
    ip_of(program)
        .file_include_reasons
        .values()
        .flatten()
        .find(|r| Arc::as_ptr(r) as *const FileIncludeReason == reason)
        .cloned()
}

pub struct IncludeExplainingDiagnostic {
    pub file: Path,
    pub diagnostic_reason: *const FileIncludeReason,
    pub diagnostic_reason_opt: Option<Box<FileIncludeReason>>,
    pub message: Message,
    pub args: Vec<String>,
}

fn ip_of(program: &Program) -> &IncludeProcessor { ::tsox_core::fntrace::enter("ip_of"); 
    // Go: includeProcessor 在 createProgram 时创建,非 nil;Rust 侧为 Option 字段
    program
        .include_processor
        .as_ref()
        .expect("include processor")
}

