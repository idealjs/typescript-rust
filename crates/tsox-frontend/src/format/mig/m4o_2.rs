use std::cell::RefCell;
use std::sync::Arc;

use super::m4o::ListFormat;
use super::m4o_3::GetLiteralTextFlags;
use crate::ast::mig::m3e_3::OPERATOR_PRECEDENCE_LOWEST;
use crate::ast::node_data_generated::{
    is_arrow_function, is_identifier, is_keyword_kind, is_punctuation_kind, NodeData,
};
use crate::ast::node_flags::NodeFlags;
use crate::ast::node_node::Node;
use crate::ast::node_node_list::NodeList;
use crate::ast::node_source_file::SourceFile;
use crate::ast::symbol_symbol::Symbol;
use crate::ast::syntax_kind_generated::SyntaxKind;
use crate::ast::utilities_expressions::is_expression;
use crate::ast::utilities_statements::is_statement;
use crate::ast::utilities_types::{is_jsdoc_kind, is_type_node};
use crate::format::mig::m4t_3::LineCharacterCache;
use crate::scanner::CommentRange;
use tsox_core::core::compiler_options_kinds::{NewLineKind, ScriptTarget};
use tsox_core::core::text::TextRange;

pub type NodeId = u64;
pub type AutoGenerateId = u32;
pub type SourceIndex = i32;
pub type EmitFlags = i32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TempFlags(pub i32);

impl TempFlags {
    pub const AUTO: TempFlags = TempFlags(1);
    pub const RESERVED_IN_NESTED_SCOPE: TempFlags = TempFlags(2);
    pub const OPTIMISTIC: TempFlags = TempFlags(4);
    pub const FILE_LEVEL: TempFlags = TempFlags(8);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WriteKind {
    #[default]
    None,
    Keyword,
    Operator,
    Punctuation,
    StringLiteral,
    Parameter,
    Property,
    Comment,
    Literal,
}

#[derive(Clone, Default)]
pub struct EmitContext;

#[derive(Clone)]
pub struct Generator;

#[derive(Clone)]
pub struct EmitTextWriter {
    state: Arc<RefCell<TextWriterState>>,
}

struct TextWriterState {
    new_line: String,
    indent: usize,
    indent_string: String,
    text: String,
}

impl EmitTextWriter {
    fn with_state<R>(&self, f: impl FnOnce(&mut TextWriterState) -> R) -> R { ::tsox_core::fntrace::enter("with_state"); 
        let mut state = self.state.borrow_mut();
        f(&mut state)
    }

    pub fn increase_indent(&self) { ::tsox_core::fntrace::enter("increase_indent"); 
        self.with_state(|s| {
            s.indent += 1;
            s.indent_string = std::iter::repeat(' ').take(s.indent * 4).collect();
        });
    }

    pub fn decrease_indent(&self) { ::tsox_core::fntrace::enter("decrease_indent"); 
        self.with_state(|s| {
            s.indent = s.indent.saturating_sub(1);
            s.indent_string = std::iter::repeat(' ').take(s.indent * 4).collect();
        });
    }

    pub fn write(&self, s: &str) { ::tsox_core::fntrace::enter("write"); 
        self.with_state(|s2| s2.text.push_str(s));
    }

    pub fn write_line(&self, s: &str) { ::tsox_core::fntrace::enter("write_line"); 
        self.with_state(|s2| {
            s2.text.push_str(&s2.indent_string);
            s2.text.push_str(s);
            s2.text.push_str(&s2.new_line);
        });
    }

    pub fn clear(&self) { ::tsox_core::fntrace::enter("clear"); 
        self.with_state(|s| s.text.clear());
    }

    pub fn string(&self) -> String { ::tsox_core::fntrace::enter("string"); 
        self.with_state(|s| s.text.clone())
    }

    pub fn grow(&self, len: usize) { ::tsox_core::fntrace::enter("grow"); 
        self.with_state(|s| s.text.reserve(len));
    }
}

pub fn new_text_writer(new_line: String, initial_indent: usize) -> EmitTextWriter { ::tsox_core::fntrace::enter("new_text_writer"); 
    EmitTextWriter {
        state: Arc::new(RefCell::new(TextWriterState {
            new_line,
            indent: initial_indent,
            indent_string: String::new(),
            text: String::new(),
        })),
    }
}

pub fn get_trailing_semicolon_deferring_writer(writer: EmitTextWriter) -> EmitTextWriter { ::tsox_core::fntrace::enter("get_trailing_semicolon_deferring_writer"); 
    writer
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeneratedIdentifierFlags(pub i32);

impl GeneratedIdentifierFlags {
    pub const NONE: GeneratedIdentifierFlags = GeneratedIdentifierFlags(0);
    pub const AUTO: GeneratedIdentifierFlags = GeneratedIdentifierFlags(1);
    pub const LOOP: GeneratedIdentifierFlags = GeneratedIdentifierFlags(2);
    pub const UNIQUE: GeneratedIdentifierFlags = GeneratedIdentifierFlags(3);
    pub const NODE: GeneratedIdentifierFlags = GeneratedIdentifierFlags(4);
    pub const KIND_MASK: GeneratedIdentifierFlags = GeneratedIdentifierFlags(7);
    pub const RESERVED_IN_NESTED_SCOPES: GeneratedIdentifierFlags = GeneratedIdentifierFlags(1 << 3);
    pub const OPTIMISTIC: GeneratedIdentifierFlags = GeneratedIdentifierFlags(1 << 4);
    pub const FILE_LEVEL: GeneratedIdentifierFlags = GeneratedIdentifierFlags(1 << 5);
    pub const ALLOW_NAME_SUBSTITUTION: GeneratedIdentifierFlags = GeneratedIdentifierFlags(1 << 6);

    pub fn kind(self) -> GeneratedIdentifierFlags { ::tsox_core::fntrace::enter("kind"); 
        GeneratedIdentifierFlags(self.0 & Self::KIND_MASK.0)
    }

    pub fn is_auto(self) -> bool { ::tsox_core::fntrace::enter("is_auto"); 
        self.kind() == Self::AUTO
    }

    pub fn is_loop(self) -> bool { ::tsox_core::fntrace::enter("is_loop"); 
        self.kind() == Self::LOOP
    }

    pub fn is_unique(self) -> bool { ::tsox_core::fntrace::enter("is_unique"); 
        self.kind() == Self::UNIQUE
    }

    pub fn is_node(self) -> bool { ::tsox_core::fntrace::enter("is_node"); 
        self.kind() == Self::NODE
    }

    pub fn is_reserved_in_nested_scopes(self) -> bool { ::tsox_core::fntrace::enter("is_reserved_in_nested_scopes"); 
        self.0 & Self::RESERVED_IN_NESTED_SCOPES.0 != 0
    }

    pub fn is_optimistic(self) -> bool { ::tsox_core::fntrace::enter("is_optimistic"); 
        self.0 & Self::OPTIMISTIC.0 != 0
    }

    pub fn is_file_level(self) -> bool { ::tsox_core::fntrace::enter("is_file_level"); 
        self.0 & Self::FILE_LEVEL.0 != 0
    }

    pub fn has_allow_name_substitution(self) -> bool { ::tsox_core::fntrace::enter("has_allow_name_substitution"); 
        self.0 & Self::ALLOW_NAME_SUBSTITUTION.0 != 0
    }
}

pub struct Priority {
    pub value: i32,
}

pub struct EmitHelper {
    pub name: String,
    pub scoped: bool,
    pub text: String,
    pub text_callback: Option<Box<dyn Fn(&dyn Fn(&str) -> String) -> String>>,
    pub priority: Option<Priority>,
    pub dependencies: Vec<Arc<EmitHelper>>,
    pub import_name: String,
}

unsafe impl Send for EmitHelper {}
unsafe impl Sync for EmitHelper {}

pub fn compare_emit_helpers(x: &EmitHelper, y: &EmitHelper) -> i32 { ::tsox_core::fntrace::enter("compare_emit_helpers"); 
    if std::ptr::eq(x, y) {
        return 0;
    }
    match (&x.priority, &y.priority) {
        (None, None) => 0,
        (Some(px), Some(py)) => px.value - py.value,
        (None, Some(_)) => 1,
        (Some(_), None) => -1,
    }
}

pub struct NameGenerator {
    pub context: EmitContext,
    pub is_file_level_unique_name_in_current_file: Option<fn(&Printer, &str, bool) -> bool>,
    pub get_text_of_node: fn(&Printer, &Arc<Node>, bool) -> String,
    pub node_id_to_generated_name: Option<std::collections::HashMap<NodeId, String>>,
    pub node_id_to_generated_private_name: Option<std::collections::HashMap<NodeId, String>>,
    pub auto_generated_id_to_generated_name:
        Option<std::collections::HashMap<AutoGenerateId, String>>,
    pub name_generation_scope: Option<Box<NameGenerationScope>>,
    pub private_name_generation_scope: Option<Box<NameGenerationScope>>,
    pub generated_names: std::collections::HashSet<String>,
}

pub struct NameGenerationScope {
    pub next: Option<Box<NameGenerationScope>>,
    pub temp_flags: TempFlags,
    pub formatted_name_temp_flags: std::collections::HashMap<String, TempFlags>,
    pub reserved_names: std::collections::HashSet<String>,
}

impl Default for NameGenerator {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        NameGenerator {
            context: EmitContext,
            is_file_level_unique_name_in_current_file: None,
            get_text_of_node: |_printer, _node, _include_trivia| String::new(),
            node_id_to_generated_name: None,
            node_id_to_generated_private_name: None,
            auto_generated_id_to_generated_name: None,
            name_generation_scope: None,
            private_name_generation_scope: None,
            generated_names: std::collections::HashSet::new(),
        }
    }
}

impl NameGenerator {
    pub fn get_scope(&mut self, private_name: bool) -> &mut Option<Box<NameGenerationScope>> { ::tsox_core::fntrace::enter("get_scope"); 
        if private_name {
            &mut self.private_name_generation_scope
        } else {
            &mut self.name_generation_scope
        }
    }

    pub fn generate_name_for_class_expression(&mut self, printer: &Printer) -> String { ::tsox_core::fntrace::enter("generate_name_for_class_expression"); 
        self.make_unique_name(printer, "class", None, false, false, false, "", "")
    }

    pub fn make_file_level_optimistic_unique_name(
        &mut self,
        printer: &Printer,
        name: &str,
    ) -> String { ::tsox_core::fntrace::enter("make_file_level_optimistic_unique_name"); 
        let check_fn = self.is_file_level_unique_name_in_current_file;
        self.make_unique_name(printer, name, check_fn, true, false, false, "", "")
    }

    fn check_unique_name(
        &self,
        printer: &Printer,
        name: &str,
        private_name: bool,
        check_fn: Option<fn(&Printer, &str, bool) -> bool>,
    ) -> bool { ::tsox_core::fntrace::enter("check_unique_name"); 
        match check_fn {
            Some(check_fn) => check_fn(printer, name, private_name),
            None => self.is_unique_name(printer, name, private_name),
        }
    }

    fn is_unique_name(&self, printer: &Printer, name: &str, private_name: bool) -> bool { ::tsox_core::fntrace::enter("is_unique_name"); 
        let is_file_level_unique = self
            .is_file_level_unique_name_in_current_file
            .map(|check| check(printer, name, private_name))
            .unwrap_or(true);
        is_file_level_unique && !self.is_reserved_name(name, private_name)
    }

    fn is_reserved_name(&self, name: &str, private_name: bool) -> bool { ::tsox_core::fntrace::enter("is_reserved_name"); 
        if self.generated_names.contains(name) {
            return true;
        }
        let mut scope = if private_name {
            self.private_name_generation_scope.as_deref()
        } else {
            self.name_generation_scope.as_deref()
        };
        while let Some(current) = scope {
            if current.reserved_names.contains(name) {
                return true;
            }
            scope = current.next.as_deref();
        }
        false
    }

    fn reserve_name(&mut self, name: &str, private_name: bool, scoped: bool, temp: bool) { ::tsox_core::fntrace::enter("reserve_name"); 
        if private_name || scoped {
            if let Some(scope) = self.get_scope(private_name).as_deref_mut() {
                scope.reserved_names.insert(name.to_string());
            } else {
                self.get_scope(private_name)
                    .replace(Box::new(NameGenerationScope {
                        next: None,
                        temp_flags: TempFlags::AUTO,
                        formatted_name_temp_flags: std::collections::HashMap::new(),
                        reserved_names: std::collections::HashSet::from([name.to_string()]),
                    }));
            }
        } else if !temp {
            self.generated_names.insert(name.to_string());
        }
    }

    pub fn make_unique_name(
        &mut self,
        printer: &Printer,
        base_name: &str,
        check_fn: Option<fn(&Printer, &str, bool) -> bool>,
        optimistic: bool,
        scoped: bool,
        private_name: bool,
        prefix: &str,
        suffix: &str,
    ) -> String { ::tsox_core::fntrace::enter("make_unique_name"); 
        let mut base_name = base_name.trim_start_matches('#').to_string();
        if optimistic {
            let full_name = format_generated_name(private_name, prefix, &base_name, suffix);
            if self.check_unique_name(printer, &full_name, private_name, check_fn) {
                self.reserve_name(&full_name, private_name, scoped, false);
                return full_name;
            }
        }

        if !base_name.is_empty() && !base_name.ends_with('_') {
            base_name.push('_');
        }

        let mut i = 1;
        loop {
            let full_name =
                format_generated_name(private_name, prefix, &format!("{base_name}{i}"), suffix);
            if self.check_unique_name(printer, &full_name, private_name, check_fn) {
                self.reserve_name(&full_name, private_name, scoped, false);
                return full_name;
            }
            i += 1;
        }
    }
}

fn format_generated_name(private_name: bool, prefix: &str, base_name: &str, suffix: &str) -> String { ::tsox_core::fntrace::enter("format_generated_name"); 
    let mut name = String::new();
    if private_name {
        name.push('#');
    }
    name.push_str(prefix);
    name.push_str(base_name);
    name.push_str(suffix);
    if private_name && prefix.is_empty() {
        name = format!("#{name}");
    }
    name
}

pub fn next_container(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("next_container"); 
    // Go: node.LocalsContainerData() → NextContainer; LocalsContainerData 访问器尚未接线,按 data == nil 路径返回 None
    let _ = node;
    None
}

pub struct PrinterOptions {
    pub remove_comments: bool,
    pub new_line: NewLineKind,
    pub omit_trailing_semicolon: bool,
    pub no_emit_helpers: bool,
    pub target: ScriptTarget,
    pub source_map: bool,
    pub inline_source_map: bool,
    pub inline_sources: bool,
    pub omit_brace_source_map_positions: bool,
    pub only_print_jsdoc_style: bool,
    pub never_ascii_escape: bool,
    pub preserve_source_newlines: bool,
    pub terminate_unterminated_literals: bool,
}

pub struct PrintHandlers {
    pub has_global_name: Option<fn(&str) -> bool>,
    pub map_source_position:
        Option<fn(Arc<SourceFile>, usize) -> Option<(Arc<SourceFile>, usize)>>,
    pub on_before_emit_node: Option<Box<dyn Fn(Option<&Arc<Node>>)>>,
    pub on_after_emit_node: Option<Box<dyn Fn(Option<&Arc<Node>>)>>,
    pub on_before_emit_node_list: Option<Box<dyn Fn(Option<&NodeList>)>>,
    pub on_after_emit_node_list: Option<Box<dyn Fn(Option<&NodeList>)>>,
    pub on_before_emit_token: Option<Box<dyn Fn(Option<&Arc<Node>>)>>,
    pub on_after_emit_token: Option<Box<dyn Fn(Option<&Arc<Node>>)>>,
}

pub struct Printer {
    pub print_handlers: PrintHandlers,
    pub options: PrinterOptions,
    pub(crate) emit_context: EmitContext,
    pub(crate) current_source_file: Option<Arc<SourceFile>>,
    pub(crate) unique_helper_names: Option<std::collections::HashMap<String, Arc<Node>>>,
    external_helpers_module_name: Option<Arc<Node>>,
    next_list_element_pos: usize,
    pub(crate) writer: Option<EmitTextWriter>,
    own_writer: Option<EmitTextWriter>,
    write_kind: WriteKind,
    source_maps_disabled: bool,
    source_map_generator: Option<Generator>,
    source_map_source: Option<Arc<SourceFile>>,
    source_map_source_index: SourceIndex,
    source_map_source_is_json: bool,
    source_map_line_char_cache: Option<Box<LineCharacterCache>>,
    most_recent_source_map_source: Option<Arc<SourceFile>>,
    most_recent_source_map_source_index: SourceIndex,
    pub(crate) container_pos: i64,
    pub(crate) container_end: i64,
    pub(crate) declaration_list_container_end: i64,
    detached_comments_info: Vec<DetachedCommentsInfo>,
    pub(crate) comments_disabled: bool,
    in_extends: bool,
    name_generator: NameGenerator,
    make_file_level_optimistic_unique_name: Option<fn(&str) -> String>,
    id_to_symbol: Option<std::collections::HashMap<Arc<Node>, Arc<Symbol>>>,
}

pub struct DetachedCommentsInfo {
    pub(crate) node_pos: usize,
    pub(crate) detached_comment_end_pos: usize,
}

pub struct CommentState {
    pub(crate) emit_flags: EmitFlags,
    pub(crate) comment_range: TextRange,
    pub(crate) container_pos: i64,
    pub(crate) container_end: i64,
    pub(crate) declaration_list_container_end: i64,
}

pub struct SourceMapState {
    emit_flags: EmitFlags,
    source_map_range: TextRange,
    has_token_source_map_range: bool,
}

pub fn new_printer(options: PrinterOptions, handlers: PrintHandlers, emit_context: EmitContext) -> Printer { ::tsox_core::fntrace::enter("new_printer"); 
    let mut printer = Printer {
        print_handlers: handlers,
        options,
        emit_context: emit_context,
        current_source_file: None,
        unique_helper_names: None,
        external_helpers_module_name: None,
        next_list_element_pos: 0,
        writer: None,
        own_writer: None,
        write_kind: WriteKind::default(),
        source_maps_disabled: false,
        source_map_generator: None,
        source_map_source: None,
        source_map_source_index: -1,
        source_map_source_is_json: false,
        source_map_line_char_cache: None,
        most_recent_source_map_source: None,
        most_recent_source_map_source_index: -1,
        container_pos: -1,
        container_end: -1,
        declaration_list_container_end: -1,
        detached_comments_info: Vec::new(),
        comments_disabled: false,
        in_extends: false,
        name_generator: NameGenerator::default(),
        make_file_level_optimistic_unique_name: None,
        id_to_symbol: None,
    };
    printer.name_generator.context = printer.emit_context.clone();
    printer.name_generator.get_text_of_node = Printer::get_text_of_node;
    printer.name_generator.is_file_level_unique_name_in_current_file =
        Some(Printer::is_file_level_unique_name_in_current_file);
    printer.comments_disabled = printer.options.remove_comments;
    printer
}

impl Printer {
    pub fn decrease_indent(&mut self) { ::tsox_core::fntrace::enter("decrease_indent"); 
        if let Some(writer) = self.writer.as_ref() {
            writer.decrease_indent();
        }
    }

    pub fn decrease_indent_if(&mut self, indent_requested: bool) { ::tsox_core::fntrace::enter("decrease_indent_if"); 
        if indent_requested {
            self.decrease_indent();
        }
    }

    pub fn comment_will_emit_new_line(&self, comment: CommentRange) -> bool { ::tsox_core::fntrace::enter("comment_will_emit_new_line"); 
        comment.kind == crate::scanner::is_jsx_line_break::CommentRangeKind::SingleLine
            || comment.has_trailing_new_line
    }

    pub fn emit(&mut self, node: &Arc<Node>, source_file: Option<&Arc<SourceFile>>) -> String { ::tsox_core::fntrace::enter("emit"); 
        if self.own_writer.is_none() {
            self.own_writer = Some(new_text_writer(
                self.options.new_line.get_new_line_character().to_string(),
                0,
            ));
        }

        let own_writer = self.own_writer.clone().expect("own writer initialized");
        self.write(node, source_file, own_writer, None);
        let text = self
            .own_writer
            .as_ref()
            .map(|w| w.string())
            .unwrap_or_default();

        if let Some(writer) = self.own_writer.as_ref() {
            writer.clear();
        }
        text
    }

    pub fn emit_source_file(&mut self, source_file: &Arc<SourceFile>) -> String { ::tsox_core::fntrace::enter("emit_source_file"); 
        self.emit(&source_file.node, Some(source_file))
    }

    pub fn write(
        &mut self,
        node: &Arc<Node>,
        source_file: Option<&Arc<SourceFile>>,
        writer: EmitTextWriter,
        source_map_generator: Option<&mut Generator>,
    ) { ::tsox_core::fntrace::enter("write"); 
        let saved_current_source_file = self.current_source_file.clone();
        let saved_writer = self.writer.clone();
        let saved_unique_helper_names = self.unique_helper_names.clone();
        let saved_source_maps_disabled = self.source_maps_disabled;
        let saved_source_map_generator = self.source_map_generator.clone();
        let saved_source_map_source = self.source_map_source.clone();
        let saved_source_map_source_index = self.source_map_source_index;
        let saved_source_map_line_char_cache = self.source_map_line_char_cache.take();

        self.source_maps_disabled = source_map_generator.is_none();
        self.source_map_generator = source_map_generator.cloned();
        self.source_map_source = None;
        self.source_map_source_index = -1;
        self.source_map_line_char_cache = None;

        self.set_source_file(source_file);
        let mut writer = writer;
        if self.options.omit_trailing_semicolon {
            writer = get_trailing_semicolon_deferring_writer(writer);
        }
        self.writer = Some(writer);
        if let Some(writer) = self.writer.as_ref() {
            writer.clear();
        }
        if let Some(writer) = self.writer.as_ref() {
            if let Some(source_file) = source_file {
                writer.grow(source_file.text.len());
            }
        }

        match node.kind {
            SyntaxKind::TemplateHead => self.emit_template_head(node),
            SyntaxKind::TemplateMiddle => self.emit_template_middle(node),
            SyntaxKind::TemplateTail => self.emit_template_tail(node),

            SyntaxKind::Identifier => self.emit_identifier_name(node),
            SyntaxKind::PrivateIdentifier => self.emit_private_identifier(node),

            SyntaxKind::QualifiedName => self.emit_qualified_name(node),
            SyntaxKind::ComputedPropertyName => {
                self.emit_computed_property_name(node)
            }

            SyntaxKind::TypeParameter => self.emit_type_parameter(node),
            SyntaxKind::Parameter => self.emit_parameter(node),
            SyntaxKind::Decorator => self.emit_decorator(node),

            SyntaxKind::PropertySignature => {
                self.emit_property_signature(node)
            }
            SyntaxKind::PropertyDeclaration => {
                self.emit_property_declaration(node)
            }
            SyntaxKind::MethodSignature => {
                self.emit_method_signature(node)
            }
            SyntaxKind::MethodDeclaration => {
                self.emit_method_declaration(node)
            }
            SyntaxKind::ClassStaticBlockDeclaration => {
                self.emit_class_static_block_declaration(node)
            }
            SyntaxKind::Constructor => self.emit_constructor(node),
            SyntaxKind::GetAccessor => {
                self.emit_accessor_declaration(SyntaxKind::GetKeyword, node)
            }
            SyntaxKind::SetAccessor => {
                self.emit_accessor_declaration(SyntaxKind::SetKeyword, node)
            }
            SyntaxKind::CallSignature => self.emit_call_signature(node),
            SyntaxKind::ConstructSignature => {
                self.emit_construct_signature(node)
            }
            SyntaxKind::IndexSignature => {
                self.emit_index_signature(node)
            }

            SyntaxKind::ObjectBindingPattern => {
                self.emit_object_binding_pattern(node)
            }
            SyntaxKind::ArrayBindingPattern => {
                self.emit_array_binding_pattern(node)
            }
            SyntaxKind::BindingElement => self.emit_binding_element(node),

            SyntaxKind::TemplateSpan => self.emit_template_span(node),
            SyntaxKind::SemicolonClassElement => {
                self.emit_semicolon_class_element(node)
            }

            SyntaxKind::VariableDeclaration => {
                self.emit_variable_declaration(node)
            }
            SyntaxKind::VariableDeclarationList => {
                self.emit_variable_declaration_list(node)
            }
            SyntaxKind::ModuleBlock => self.emit_module_block(node),
            SyntaxKind::CaseBlock => self.emit_case_block(node),
            SyntaxKind::ImportClause => self.emit_import_clause(node),
            SyntaxKind::NamespaceImport => self.emit_namespace_import(node),
            SyntaxKind::NamespaceExport => self.emit_namespace_export(node),
            SyntaxKind::NamedImports => self.emit_named_imports(node),
            SyntaxKind::ImportSpecifier => self.emit_import_specifier(node),
            SyntaxKind::NamedExports => self.emit_named_exports(node),
            SyntaxKind::ExportSpecifier => self.emit_export_specifier(node),
            SyntaxKind::ImportAttributes => self.emit_import_attributes(node),
            SyntaxKind::ImportAttribute => self.emit_import_attribute(node),

            SyntaxKind::ExternalModuleReference => {
                self.emit_external_module_reference(node)
            }

            SyntaxKind::JsxText => self.emit_jsx_text(node),
            SyntaxKind::JsxOpeningElement => self.emit_jsx_opening_element(node),
            SyntaxKind::JsxOpeningFragment => {
                self.emit_jsx_opening_fragment(node)
            }
            SyntaxKind::JsxClosingElement => self.emit_jsx_closing_element(node),
            SyntaxKind::JsxClosingFragment => {
                self.emit_jsx_closing_fragment(node)
            }
            SyntaxKind::JsxAttribute => self.emit_jsx_attribute(node),
            SyntaxKind::JsxAttributes => self.emit_jsx_attributes(node),
            SyntaxKind::JsxSpreadAttribute => {
                self.emit_jsx_spread_attribute(node)
            }
            SyntaxKind::JsxExpression => self.emit_jsx_expression(node),
            SyntaxKind::JsxNamespacedName => {
                self.emit_jsx_namespaced_name(node)
            }

            SyntaxKind::CaseClause => self.emit_case_clause(node),
            SyntaxKind::DefaultClause => self.emit_default_clause(node),
            SyntaxKind::HeritageClause => self.emit_heritage_clause_node(node),
            SyntaxKind::CatchClause => self.emit_catch_clause(node),

            SyntaxKind::PropertyAssignment => {
                self.emit_property_assignment(node)
            }
            SyntaxKind::ShorthandPropertyAssignment => {
                self.emit_shorthand_property_assignment(node)
            }
            SyntaxKind::SpreadAssignment => self.emit_spread_assignment(node),

            SyntaxKind::EnumMember => self.emit_enum_member(node),

            SyntaxKind::SourceFile => self.emit_source_file_node(node),

            SyntaxKind::NotEmittedTypeElement => {
                self.emit_not_emitted_type_element(node)
            }

            _ => {
                if is_type_node(node) {
                    self.emit_type_node_outside_extends(node);
                } else if is_statement(node) {
                    self.emit_statement(node);
                } else if is_expression(node) {
                    self.emit_expression(node, OPERATOR_PRECEDENCE_LOWEST);
                } else if is_keyword_kind(node.kind) {
                    self.emit_keyword_node(node);
                } else if is_punctuation_kind(node.kind) {
                    self.emit_punctuation_node(node);
                } else if is_jsdoc_kind(node.kind) {
                    self.emit_jsdoc_node(node);
                } else {
                    panic!("unhandled Node: {:?}", node.kind);
                }
            }
        }

        self.current_source_file = saved_current_source_file;
        self.writer = saved_writer;
        self.unique_helper_names = saved_unique_helper_names;
        self.source_maps_disabled = saved_source_maps_disabled;
        self.source_map_generator = saved_source_map_generator;
        self.source_map_source = saved_source_map_source;
        self.source_map_source_index = saved_source_map_source_index;
        self.source_map_line_char_cache = saved_source_map_line_char_cache;
    }

    pub(crate) fn get_text_of_node(&self, node: &Arc<Node>, _include_trivia: bool) -> String { ::tsox_core::fntrace::enter("get_text_of_node"); 
        crate::scanner::mig::m3i::get_text_of_node(node)
    }

    pub(crate) fn is_file_level_unique_name_in_current_file(
        &self,
        name: &str,
        _private_name: bool,
    ) -> bool { ::tsox_core::fntrace::enter("is_file_level_unique_name_in_current_file"); 
        match &self.current_source_file {
            Some(source_file) => self.emit_context.is_file_level_unique_name(
                source_file,
                name,
                self.print_handlers.has_global_name,
            ),
            None => true,
        }
    }

    pub(crate) fn set_source_file(&mut self, source_file: Option<&Arc<SourceFile>>) { ::tsox_core::fntrace::enter("set_source_file"); 
        self.current_source_file = source_file.cloned();
        self.unique_helper_names = None;
        self.external_helpers_module_name = None;
    }
}

pub fn can_emit_simple_arrow_head(parent_node: &Arc<Node>, parameters: &NodeList) -> bool { ::tsox_core::fntrace::enter("can_emit_simple_arrow_head"); 
    if !is_arrow_function(parent_node) || parameters.nodes.len() != 1 {
        return false;
    }

    let parameter = &parameters.nodes[0];

    parameter.loc == parent_node.loc
        && parent_node.type_parameters().is_none()
        && parent_node.type_().is_none()
        && parent_node
            .modifiers()
            .map(|m| m.nodes.is_empty())
            .unwrap_or(true)
        && !parameters.has_trailing_comma()
        && parameter.modifiers().is_none()
        && parameter.dot_dot_dot_token().is_none()
        && match &parameter.data {
            NodeData::ParameterDeclaration(d) => d.question_token.is_none(),
            _ => true,
        }
        && parameter.type_().is_none()
        && parameter.initializer().is_none()
        && parameter.name().map(|n| is_identifier(n)).unwrap_or(false)
}

impl EmitContext {
    pub fn is_file_level_unique_name(
        &self,
        _source_file: &SourceFile,
        name: &str,
        has_global_name: Option<fn(&str) -> bool>,
    ) -> bool { ::tsox_core::fntrace::enter("is_file_level_unique_name"); 
        if has_global_name.map(|check| check(name)).unwrap_or(false) {
            return false;
        }
        true
    }
}

fn lf_named_imports_or_exports_elements() -> ListFormat { ::tsox_core::fntrace::enter("lf_named_imports_or_exports_elements"); 
    ListFormat::COMMA_DELIMITED
        | ListFormat::SPACE_BETWEEN_SIBLINGS
        | ListFormat::ALLOW_TRAILING_COMMA
        | ListFormat::SPACE_BETWEEN_BRACES
        | ListFormat::NO_SPACE_IF_EMPTY
}

fn lf_import_attributes() -> ListFormat { ::tsox_core::fntrace::enter("lf_import_attributes"); 
    ListFormat::COMMA_DELIMITED | ListFormat::ALLOW_TRAILING_COMMA
}

fn lf_jsx_element_attributes() -> ListFormat { ::tsox_core::fntrace::enter("lf_jsx_element_attributes"); 
    ListFormat::SPACE_BETWEEN_SIBLINGS | ListFormat::NO_INTERVENING_COMMENTS
}

fn lf_jsx_element_or_fragment_children() -> ListFormat { ::tsox_core::fntrace::enter("lf_jsx_element_or_fragment_children"); 
    ListFormat::NO_INTERVENING_COMMENTS
}

impl Printer {
    fn emit_string_literal(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_string_literal"); 
        let state = self.enter_node(node);
        self.emit_literal(node, GetLiteralTextFlags::NONE);
        self.exit_node(node, state);
    }

    fn emit_module_export_name(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_module_export_name"); 
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_name(node),
            SyntaxKind::StringLiteral => self.emit_string_literal(node),
            _ => panic!("unexpected ModuleExportName: {:?}", node.kind),
        }
    }

    fn emit_import_attribute_name(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_import_attribute_name"); 
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_name(node),
            SyntaxKind::StringLiteral => self.emit_string_literal(node),
            _ => panic!("unexpected ImportAttributeName: {:?}", node.kind),
        }
    }

    fn emit_template_middle_tail(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_template_middle_tail"); 
        match node.kind {
            SyntaxKind::TemplateMiddle => self.emit_template_middle(node),
            SyntaxKind::TemplateTail => self.emit_template_tail(node),
            _ => {}
        }
    }

    pub fn emit_template_head(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_template_head"); 
        let state = self.enter_node(node);
        self.emit_literal(node, GetLiteralTextFlags::NONE);
        self.exit_node(node, state);
    }

    pub fn emit_template_middle(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_template_middle"); 
        let state = self.enter_node(node);
        self.emit_literal(node, GetLiteralTextFlags::NONE);
        self.exit_node(node, state);
    }

    pub fn emit_template_tail(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_template_tail"); 
        let state = self.enter_node(node);
        self.emit_literal(node, GetLiteralTextFlags::NONE);
        self.exit_node(node, state);
    }

    pub fn emit_private_identifier(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_private_identifier"); 
        let state = self.enter_node(node);
        let text = self.get_text_of_node(node, false);
        self.write_as(&text, WriteKind::None);
        self.exit_node(node, state);
    }

    pub fn emit_qualified_name(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_qualified_name"); 
        let state = self.enter_node(node);
        let NodeData::QualifiedName(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        match d.left.kind {
            SyntaxKind::Identifier => self.emit_identifier_name(&d.left),
            SyntaxKind::QualifiedName => self.emit_qualified_name(&d.left),
            _ => self.emit_expression(&d.left, OPERATOR_PRECEDENCE_LOWEST),
        }
        self.write_punctuation(".");
        match d.right.kind {
            SyntaxKind::Identifier => self.emit_identifier_name(&d.right),
            SyntaxKind::PrivateIdentifier => self.emit_private_identifier(&d.right),
            _ => self.emit_identifier_name(&d.right),
        }
        self.exit_node(node, state);
    }

    pub fn emit_type_parameter(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_type_parameter"); 
        let state = self.enter_node(node);
        self.emit_modifier_list(node, node.modifiers(), false);
        let NodeData::TypeParameterDeclaration(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        self.emit_binding_identifier(&d.name);
        if let Some(constraint) = d.constraint.as_ref() {
            self.write_space();
            self.write_keyword("extends");
            self.write_space();
            self.emit_type_node_outside_extends(constraint);
        }
        if let Some(default_type) = d.default_type.as_ref() {
            self.write_space();
            self.write_punctuation("=");
            self.write_space();
            self.emit_type_node_outside_extends(default_type);
        }
        self.exit_node(node, state);
    }

    pub fn emit_property_signature(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_property_signature"); 
        let state = self.enter_node(node);
        let NodeData::PropertySignatureDeclaration(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        self.emit_modifier_list(node, node.modifiers(), false);
        self.emit_property_name(Some(&d.name));
        self.emit_token_node(d.postfix_token.as_ref());
        self.emit_type_annotation(Some(&d.type_node));
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub fn emit_method_signature(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_method_signature"); 
        let state = self.enter_node(node);
        let NodeData::MethodSignatureDeclaration(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        self.emit_modifier_list(node, node.modifiers(), false);
        self.emit_property_name(Some(&d.name));
        self.emit_token_node(d.postfix_token.as_ref());
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(node);
        if let Some(type_parameters) = d.type_parameters.as_ref() {
            self.emit_type_parameters(node, &type_parameters.nodes);
        }
        self.emit_parameters(node, &d.parameters.nodes);
        self.emit_return_type(d.type_node.as_ref());
        self.write_trailing_semicolon();
        self.pop_name_generation_scope(node);
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub fn emit_template_span(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_template_span"); 
        let state = self.enter_node(node);
        let NodeData::TemplateSpan(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        self.emit_expression(&d.expression, OPERATOR_PRECEDENCE_LOWEST);
        self.emit_template_middle_tail(&d.literal);
        self.exit_node(node, state);
    }

    pub fn emit_variable_declaration_list(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_variable_declaration_list"); 
        let state = self.enter_node(node);
        let scoped = node.flags & NodeFlags::BlockScoped;
        if scoped == NodeFlags::Let {
            self.write_keyword("let");
        } else if scoped == NodeFlags::Const {
            self.write_keyword("const");
        } else if scoped == NodeFlags::Using {
            self.write_keyword("using");
        } else if scoped == NodeFlags::AwaitUsing {
            self.write_keyword("await");
            self.write_space();
            self.write_keyword("using");
        } else {
            self.write_keyword("var");
        }
        self.write_space();
        if let NodeData::VariableDeclarationList(d) = &node.data {
            for (i, declaration) in d.declarations.nodes.iter().enumerate() {
                if i > 0 {
                    self.write_punctuation(",");
                    self.write_space();
                }
                self.emit_variable_declaration(declaration);
            }
        }
        self.exit_node(node, state);
    }

    pub fn emit_module_block(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_module_block"); 
        let state = self.enter_node(node);
        self.generate_names(node);
        self.emit_token(
            SyntaxKind::OpenBraceToken,
            node.pos(),
            WriteKind::Punctuation,
            node,
        );
        let statements: &[Arc<Node>] = match &node.data {
            NodeData::ModuleBlock(d) => &d.statements.nodes,
            _ => &[] as &[Arc<Node>],
        };
        let format = if self.is_empty_block(node, statements) || self.should_emit_on_single_line(node)
        {
            ListFormat::SINGLE_LINE_BLOCK_STATEMENTS
        } else {
            ListFormat::MULTI_LINE_BLOCK_STATEMENTS
        };
        self.emit_list(Printer::emit_statement, node, statements, format);
        self.emit_token(
            SyntaxKind::CloseBraceToken,
            statements.last().map(|s| s.end()).unwrap_or_else(|| node.pos()),
            WriteKind::Punctuation,
            node,
        );
        self.exit_node(node, state);
    }

    pub fn emit_import_clause(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_import_clause"); 
        let state = self.enter_node(node);
        let NodeData::ImportClause(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        if let Some(phase_modifier) = d.phase_modifier {
            self.emit_token(phase_modifier, node.pos(), WriteKind::Keyword, node);
            self.write_space();
        }
        if let Some(name) = d.name.as_ref() {
            self.emit_binding_identifier(name);
            if d.named_bindings.is_some() {
                self.emit_token(SyntaxKind::CommaToken, name.end(), WriteKind::Punctuation, node);
                self.write_space();
            }
        }
        if let Some(bindings) = d.named_bindings.as_ref() {
            match bindings.kind {
                SyntaxKind::NamespaceImport => self.emit_namespace_import(bindings),
                SyntaxKind::NamedImports => self.emit_named_imports(bindings),
                _ => panic!("unhandled NamedImportBindings: {:?}", bindings.kind),
            }
        }
        self.exit_node(node, state);
    }

    pub fn emit_namespace_import(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_namespace_import"); 
        let state = self.enter_node(node);
        let NodeData::NamespaceImport(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        let pos = self.emit_token(
            SyntaxKind::AsteriskToken,
            node.pos(),
            WriteKind::Punctuation,
            node,
        );
        self.write_space();
        self.emit_token(SyntaxKind::AsKeyword, pos, WriteKind::Keyword, node);
        self.write_space();
        self.emit_binding_identifier(&d.name);
        self.exit_node(node, state);
    }

    pub fn emit_namespace_export(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_namespace_export"); 
        let state = self.enter_node(node);
        let NodeData::NamespaceExport(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        let pos = self.emit_token(
            SyntaxKind::AsteriskToken,
            node.pos(),
            WriteKind::Punctuation,
            node,
        );
        self.write_space();
        self.emit_token(SyntaxKind::AsKeyword, pos, WriteKind::Keyword, node);
        self.write_space();
        self.emit_module_export_name(&d.name);
        self.exit_node(node, state);
    }

    pub fn emit_named_imports(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_named_imports"); 
        let state = self.enter_node(node);
        let elements = match &node.data {
            NodeData::NamedImports(d) => &d.elements.nodes,
            _ => &[] as &[Arc<Node>],
        };
        self.write_punctuation("{");
        self.emit_list(
            Printer::emit_import_specifier,
            node,
            elements,
            lf_named_imports_or_exports_elements(),
        );
        self.write_punctuation("}");
        self.exit_node(node, state);
    }

    pub fn emit_import_specifier(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_import_specifier"); 
        let state = self.enter_node(node);
        let NodeData::ImportSpecifier(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        if d.is_type_only {
            self.write_keyword("type");
            self.write_space();
        }
        if let Some(property_name) = d.property_name.as_ref() {
            self.emit_module_export_name(property_name);
            self.write_space();
            self.emit_token(
                SyntaxKind::AsKeyword,
                property_name.end(),
                WriteKind::Keyword,
                node,
            );
            self.write_space();
        }
        self.emit_binding_identifier(&d.name);
        self.exit_node(node, state);
    }

    pub fn emit_named_exports(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_named_exports"); 
        let state = self.enter_node(node);
        let elements = match &node.data {
            NodeData::NamedExports(d) => &d.elements.nodes,
            _ => &[] as &[Arc<Node>],
        };
        self.write_punctuation("{");
        self.emit_list(
            Printer::emit_export_specifier,
            node,
            elements,
            lf_named_imports_or_exports_elements(),
        );
        self.write_punctuation("}");
        self.exit_node(node, state);
    }

    pub fn emit_export_specifier(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_export_specifier"); 
        let state = self.enter_node(node);
        let NodeData::ExportSpecifier(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        if d.is_type_only {
            self.write_keyword("type");
            self.write_space();
        }
        if let Some(property_name) = d.property_name.as_ref() {
            self.emit_module_export_name(property_name);
            self.write_space();
            self.emit_token(
                SyntaxKind::AsKeyword,
                property_name.end(),
                WriteKind::Keyword,
                node,
            );
            self.write_space();
        }
        self.emit_module_export_name(&d.name);
        self.exit_node(node, state);
    }

    pub fn emit_import_attributes(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_import_attributes"); 
        let state = self.enter_node(node);
        let NodeData::ImportAttributes(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        self.emit_token(d.token, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        let attributes: &[Arc<Node>] = &d.attributes.nodes;
        self.emit_list(
            Printer::emit_import_attribute,
            node,
            attributes,
            lf_import_attributes(),
        );
        self.exit_node(node, state);
    }

    pub fn emit_import_attribute(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_import_attribute"); 
        let state = self.enter_node(node);
        let NodeData::ImportAttribute(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        self.emit_import_attribute_name(&d.name);
        self.write_punctuation(":");
        self.write_space();
        self.emit_expression(&d.value, OPERATOR_PRECEDENCE_LOWEST);
        self.exit_node(node, state);
    }

    pub fn emit_external_module_reference(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_external_module_reference"); 
        let state = self.enter_node(node);
        let NodeData::ExternalModuleReference(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        self.write_keyword("require");
        self.write_punctuation("(");
        self.emit_expression(&d.expression, OPERATOR_PRECEDENCE_LOWEST);
        self.write_punctuation(")");
        self.exit_node(node, state);
    }

    pub fn emit_jsx_text(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_text"); 
        let state = self.enter_node(node);
        let text = match &node.data {
            NodeData::JsxText(d) => d.text.as_str(),
            _ => node.text(),
        };
        self.write_literal(text);
        self.exit_node(node, state);
    }

    pub fn emit_jsx_opening_element(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_opening_element"); 
        let state = self.enter_node(node);
        let NodeData::JsxOpeningElement(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        self.write_punctuation("<");
        self.emit_jsx_tag_name(&d.tag_name);
        self.emit_type_arguments(node, d.type_arguments.as_ref());
        let attribute_count = match d.attributes.data {
            NodeData::JsxAttributes(ref attributes) => attributes.properties.nodes.len(),
            _ => 0,
        };
        if attribute_count > 0 {
            self.write_space();
        }
        self.emit_jsx_attributes(&d.attributes);
        self.write_punctuation(">");
        self.exit_node(node, state);
    }

    pub fn emit_jsx_opening_fragment(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_opening_fragment"); 
        let state = self.enter_node(node);
        self.write_punctuation("<");
        self.write_punctuation(">");
        self.exit_node(node, state);
    }

    pub fn emit_jsx_closing_element(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_closing_element"); 
        let state = self.enter_node(node);
        let NodeData::JsxClosingElement(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        self.write_punctuation("</");
        self.emit_jsx_tag_name(&d.tag_name);
        self.write_punctuation(">");
        self.exit_node(node, state);
    }

    pub fn emit_jsx_closing_fragment(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_closing_fragment"); 
        let state = self.enter_node(node);
        self.write_punctuation("</");
        self.write_punctuation(">");
        self.exit_node(node, state);
    }

    pub fn emit_jsx_attribute(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_attribute"); 
        let state = self.enter_node(node);
        let NodeData::JsxAttribute(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        self.emit_jsx_attribute_name(&d.name);
        if let Some(initializer) = d.initializer.as_ref() {
            self.write_punctuation("=");
            self.emit_jsx_attribute_value(initializer);
        }
        self.exit_node(node, state);
    }

    pub fn emit_jsx_attributes(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_attributes"); 
        let state = self.enter_node(node);
        let properties = match &node.data {
            NodeData::JsxAttributes(d) => &d.properties.nodes,
            _ => &[] as &[Arc<Node>],
        };
        self.emit_list(
            Printer::emit_jsx_attribute_like,
            node,
            properties,
            lf_jsx_element_attributes(),
        );
        self.exit_node(node, state);
    }

    pub fn emit_jsx_spread_attribute(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_spread_attribute"); 
        let state = self.enter_node(node);
        let NodeData::JsxSpreadAttribute(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        self.write_punctuation("{...");
        self.emit_expression(&d.expression, OPERATOR_PRECEDENCE_LOWEST);
        self.write_punctuation("}");
        self.exit_node(node, state);
    }

    pub fn emit_jsx_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_expression"); 
        let state = self.enter_node(node);
        let NodeData::JsxExpression(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        if let Some(expression) = d.expression.as_ref() {
            self.emit_token(
                SyntaxKind::OpenBraceToken,
                node.pos(),
                WriteKind::Punctuation,
                node,
            );
            self.emit_token_node(d.dot_dot_dot_token.as_ref());
            self.emit_expression(expression, OPERATOR_PRECEDENCE_LOWEST);
            self.emit_token(
                SyntaxKind::CloseBraceToken,
                node.end(),
                WriteKind::Punctuation,
                node,
            );
        }
        self.exit_node(node, state);
    }

    pub fn emit_jsx_namespaced_name(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_namespaced_name"); 
        let state = self.enter_node(node);
        let NodeData::JsxNamespacedName(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        self.emit_identifier_name(&d.namespace);
        self.write_punctuation(":");
        self.emit_identifier_name(&d.name);
        self.exit_node(node, state);
    }

    fn emit_jsx_tag_name(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_tag_name"); 
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_name(node),
            SyntaxKind::JsxNamespacedName => self.emit_jsx_namespaced_name(node),
            SyntaxKind::ThisKeyword => self.write_as(node.text(), WriteKind::None),
            SyntaxKind::PropertyAccessExpression => {
                self.emit_expression(node, OPERATOR_PRECEDENCE_LOWEST)
            }
            _ => panic!("unhandled JsxTagName: {:?}", node.kind),
        }
    }

    fn emit_jsx_attribute_name(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_attribute_name"); 
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_name(node),
            SyntaxKind::JsxNamespacedName => self.emit_jsx_namespaced_name(node),
            _ => panic!("unhandled JsxAttributeName: {:?}", node.kind),
        }
    }

    fn emit_jsx_attribute_value(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_attribute_value"); 
        match node.kind {
            SyntaxKind::StringLiteral => self.emit_string_literal(node),
            SyntaxKind::JsxExpression => self.emit_jsx_expression(node),
            SyntaxKind::JsxElement => self.emit_expression(node, OPERATOR_PRECEDENCE_LOWEST),
            SyntaxKind::JsxSelfClosingElement => {
                self.emit_expression(node, OPERATOR_PRECEDENCE_LOWEST)
            }
            SyntaxKind::JsxFragment => self.emit_expression(node, OPERATOR_PRECEDENCE_LOWEST),
            _ => self.emit_expression(node, OPERATOR_PRECEDENCE_LOWEST),
        }
    }

    fn emit_jsx_attribute_like(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_attribute_like"); 
        match node.kind {
            SyntaxKind::JsxAttribute => self.emit_jsx_attribute(node),
            SyntaxKind::JsxSpreadAttribute => self.emit_jsx_spread_attribute(node),
            _ => panic!("unhandled JsxAttributeLike: {:?}", node.kind),
        }
    }

    pub fn emit_property_assignment(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_property_assignment"); 
        let state = self.enter_node(node);
        let NodeData::PropertyAssignment(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        self.emit_property_name(Some(&d.name));
        self.write_punctuation(":");
        self.write_space();
        self.emit_expression(&d.initializer, OPERATOR_PRECEDENCE_LOWEST);
        self.exit_node(node, state);
    }

    pub fn emit_shorthand_property_assignment(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_shorthand_property_assignment"); 
        let state = self.enter_node(node);
        let NodeData::ShorthandPropertyAssignment(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        self.emit_property_name(Some(&d.name));
        if let Some(object_assignment_initializer) = d.object_assignment_initializer.as_ref() {
            self.write_space();
            self.write_punctuation("=");
            self.write_space();
            self.emit_expression(object_assignment_initializer, OPERATOR_PRECEDENCE_LOWEST);
        }
        self.exit_node(node, state);
    }

    pub fn emit_spread_assignment(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_spread_assignment"); 
        let state = self.enter_node(node);
        let NodeData::SpreadAssignment(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        self.emit_token(
            SyntaxKind::DotDotDotToken,
            node.pos(),
            WriteKind::Punctuation,
            node,
        );
        self.emit_expression(&d.expression, OPERATOR_PRECEDENCE_LOWEST);
        self.exit_node(node, state);
    }

    pub fn emit_enum_member(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_enum_member"); 
        let state = self.enter_node(node);
        let NodeData::EnumMember(d) = &node.data else {
            self.exit_node(node, state);
            return;
        };
        self.emit_property_name(Some(&d.name));
        if let Some(initializer) = d.initializer.as_ref() {
            self.emit_initializer(Some(initializer), d.name.end(), node);
        }
        self.exit_node(node, state);
    }

    pub fn emit_source_file_node(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_source_file_node"); 
        let state = self.enter_node(node);
        if let NodeData::SourceFile(d) = &node.data {
            let statements: &[Arc<Node>] = &d.statements.nodes;
            self.emit_list(Printer::emit_statement, node, statements, ListFormat::MULTI_LINE);
        }
        self.exit_node(node, state);
    }

    pub fn emit_not_emitted_type_element(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_not_emitted_type_element"); 
        let state = self.enter_node(node);
        self.exit_node(node, state);
    }

    pub fn emit_keyword_node(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_keyword_node"); 
        let state = self.enter_node(node);
        self.write_token_text(node.kind, WriteKind::Keyword, node.pos());
        self.exit_node(node, state);
    }

    pub fn emit_jsdoc_node(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsdoc_node"); 
        let _ = node;
        panic!("not implemented");
    }
}
