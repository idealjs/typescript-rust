use crate::ast::node_data_generated::{
    for_each_child, is_binding_pattern, is_named_exports, NodeData,
};
use crate::ast::node_flags::ModifierFlags;
use crate::ast::node_node::Node;
use crate::ast::node_node_list::ModifierList;
use crate::ast::node_source_file::SourceFile;
use crate::ast::symbol_map::NodeSymbolMap;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use tsox_core::core::text::TextRange;

pub fn diagnostics(_file: &SourceFile) -> &'static [crate::ast::diagnostic::Diagnostic] { ::tsox_core::fntrace::enter("diagnostics"); 
    &[]
}

pub fn js_diagnostics(_file: &SourceFile) -> &'static [crate::ast::diagnostic::Diagnostic] { ::tsox_core::fntrace::enter("js_diagnostics"); 
    &[]
}

pub fn jsdoc_diagnostics(_file: &SourceFile) -> &'static [crate::ast::diagnostic::Diagnostic] { ::tsox_core::fntrace::enter("jsdoc_diagnostics"); 
    &[]
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceFileParseOptions {
    pub file_name: String,
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MappedDiagnosticDirectivePolicy(pub u8);

impl MappedDiagnosticDirectivePolicy {
    pub const IGNORE: MappedDiagnosticDirectivePolicy = MappedDiagnosticDirectivePolicy(0);
    pub const EXPECT: MappedDiagnosticDirectivePolicy = MappedDiagnosticDirectivePolicy(1);
}

#[derive(Debug, Clone, Default)]
pub struct MappedDiagnosticDirective {
    pub original_range: TextRange,
    pub virtual_range: TextRange,
    pub policy: MappedDiagnosticDirectivePolicy,
    pub unused_code: i32,
    pub unused_message_text: String,
    pub source: String,
}

pub fn path(file: &SourceFile) -> String { ::tsox_core::fntrace::enter("path"); 
    tsox_core::tspath::normalize_path(&file.file_name)
}

pub fn parse_options(file: &SourceFile) -> SourceFileParseOptions { ::tsox_core::fntrace::enter("parse_options"); 
    SourceFileParseOptions {
        path: tsox_core::tspath::normalize_path(&file.file_name),
        file_name: file.file_name.clone(),
    }
}

pub fn original_text(file: &SourceFile) -> &str { ::tsox_core::fntrace::enter("original_text"); 
    &file.text
}

pub fn original_file_name(file: &SourceFile) -> &str { ::tsox_core::fntrace::enter("original_file_name"); 
    &file.file_name
}

pub fn diagnostic_directives(_file: &SourceFile) -> &'static [MappedDiagnosticDirective] { ::tsox_core::fntrace::enter("diagnostic_directives"); 
    &[]
}

pub fn is_content_mapper_failure_stub(_file: &SourceFile) -> bool { ::tsox_core::fntrace::enter("is_content_mapper_failure_stub"); 
    false
}

pub fn is_content_mapper_supplemental(_file: &SourceFile) -> bool { ::tsox_core::fntrace::enter("is_content_mapper_supplemental"); 
    false
}

pub fn is_js(file: &SourceFile) -> bool { ::tsox_core::fntrace::enter("is_js"); 
    file.script_kind == crate::ast::node_source_file::ScriptKind::Js
        || file.script_kind == crate::ast::node_source_file::ScriptKind::Jsx
}

pub fn is_bound(_file: &SourceFile) -> bool { ::tsox_core::fntrace::enter("is_bound"); 
    false
}

pub fn ecma_line_map(file: &SourceFile) -> Vec<usize> { ::tsox_core::fntrace::enter("ecma_line_map"); 
    compute_ecma_line_starts(&file.text)
}

fn compute_ecma_line_starts(text: &str) -> Vec<usize> { ::tsox_core::fntrace::enter("compute_ecma_line_starts"); 
    let mut starts = vec![0];
    for (i, b) in text.bytes().enumerate() {
        if b == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

pub fn get_position_map(file: &SourceFile) -> crate::ast::positionmap::PositionMap { ::tsox_core::fntrace::enter("get_position_map"); 
    crate::ast::positionmap::compute_position_map(&file.text)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenCacheKey {
    pub parent_id: u64,
    pub loc: TextRange,
}

pub fn get_or_create_token(
    file: &SourceFile,
    kind: crate::ast::syntax_kind_generated::SyntaxKind,
    pos: usize,
    end: usize,
    parent: &Arc<Node>,
    flags: crate::ast::node_data_generated::TokenFlags,
) -> Arc<Node> { ::tsox_core::fntrace::enter("get_or_create_token"); 
    use crate::ast::node_flags::NodeFlags;
    assert!(
        !parent.flags.contains(NodeFlags::Reparsed),
        "Cannot create token from reparsed node of kind {:?}",
        parent.kind
    );
    Arc::new(create_token(kind, file, pos, end, flags))
}

fn create_token(
    kind: crate::ast::syntax_kind_generated::SyntaxKind,
    file: &SourceFile,
    pos: usize,
    end: usize,
    flags: crate::ast::node_data_generated::TokenFlags,
) -> Node { ::tsox_core::fntrace::enter("create_token"); 
    use crate::ast::node_data_generated::{
        BigIntLiteralData, IdentifierData, JsxTextData, NoSubstitutionTemplateLiteralData,
        NumericLiteralData, PrivateIdentifierData, RegularExpressionLiteralData,
        StringLiteralData, TemplateHeadData, TemplateMiddleData, TemplateTailData,
    };
    use crate::ast::syntax_kind_generated::SyntaxKind;
    let text = file.text[pos..end].to_string();
    match kind {
        SyntaxKind::NumericLiteral => Node::new(
            kind,
            NodeData::NumericLiteral(NumericLiteralData {
                text,
                token_flags: flags,
            }),
        ),
        SyntaxKind::BigIntLiteral => Node::new(
            kind,
            NodeData::BigIntLiteral(BigIntLiteralData {
                text,
                token_flags: flags,
            }),
        ),
        SyntaxKind::StringLiteral => Node::new(
            kind,
            NodeData::StringLiteral(StringLiteralData {
                text,
                token_flags: flags,
            }),
        ),
        SyntaxKind::JsxText | SyntaxKind::JsxTextAllWhiteSpaces => Node::new(
            kind,
            NodeData::JsxText(JsxTextData {
                text,
                contains_only_trivia_white_spaces: kind == SyntaxKind::JsxTextAllWhiteSpaces,
            }),
        ),
        SyntaxKind::RegularExpressionLiteral => Node::new(
            kind,
            NodeData::RegularExpressionLiteral(RegularExpressionLiteralData {
                text,
                token_flags: flags,
            }),
        ),
        SyntaxKind::NoSubstitutionTemplateLiteral => Node::new(
            kind,
            NodeData::NoSubstitutionTemplateLiteral(NoSubstitutionTemplateLiteralData {
                text,
                template_flags: flags,
            }),
        ),
        SyntaxKind::TemplateHead => Node::new(
            kind,
            NodeData::TemplateHead(TemplateHeadData {
                text,
                raw_text: String::new(),
                template_flags: flags,
            }),
        ),
        SyntaxKind::TemplateMiddle => Node::new(
            kind,
            NodeData::TemplateMiddle(TemplateMiddleData {
                text,
                raw_text: String::new(),
                template_flags: flags,
            }),
        ),
        SyntaxKind::TemplateTail => Node::new(
            kind,
            NodeData::TemplateTail(TemplateTailData {
                text,
                raw_text: String::new(),
                template_flags: flags,
            }),
        ),
        SyntaxKind::Identifier => Node::new(kind, NodeData::Identifier(IdentifierData { text })),
        SyntaxKind::PrivateIdentifier => Node::new(
            kind,
            NodeData::PrivateIdentifier(PrivateIdentifierData { text }),
        ),
        _ => Node::new(kind, NodeData::Token),
    }
}

static SOURCE_FILE_DATA_KEY_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct SourceFileDataKey<T> {
    key: u64,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> SourceFileDataKey<T> {
    pub fn new() -> Self { ::tsox_core::fntrace::enter("new"); 
        Self {
            key: SOURCE_FILE_DATA_KEY_COUNTER.fetch_add(1, Ordering::Relaxed) + 1,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Default for SourceFileDataKey<T> {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        Self::new()
    }
}

#[derive(Default)]
pub struct SourceFileDataStore {
    cells: Mutex<HashMap<u64, Arc<Mutex<Option<Box<dyn Any + Send>>>>>>,
}

impl SourceFileDataStore {
    fn cell_of(&self, key: u64) -> Arc<Mutex<Option<Box<dyn Any + Send>>>> { ::tsox_core::fntrace::enter("cell_of"); 
        let mut cells = self.cells.lock().unwrap();
        cells.entry(key).or_insert_with(|| Arc::new(Mutex::new(None))).clone()
    }
}

pub fn get_or_compute_source_file_data<T, F>(
    file: &SourceFile,
    _key: &SourceFileDataKey<T>,
    compute: F,
) -> T
where
    T: Clone + Any + Send + 'static,
    F: FnOnce(&SourceFile) -> T,
{ ::tsox_core::fntrace::enter("get_or_compute_source_file_data"); 
    assert!(_key.key != 0, "invalid SourceFileDataKey; use new()");
    compute(file)
}

pub struct NodeFactory {
    node_count: AtomicU64,
    text_count: AtomicU64,
}

impl NodeFactory {
    pub fn new() -> Self { ::tsox_core::fntrace::enter("new"); 
        Self {
            node_count: AtomicU64::new(0),
            text_count: AtomicU64::new(0),
        }
    }

    pub fn node_count(&self) -> u64 { ::tsox_core::fntrace::enter("node_count"); 
        self.node_count.load(Ordering::Relaxed)
    }

    pub fn text_count(&self) -> u64 { ::tsox_core::fntrace::enter("text_count"); 
        self.text_count.load(Ordering::Relaxed)
    }

    pub fn new_node_list(&self, nodes: Vec<Arc<Node>>) -> crate::ast::node_node_list::NodeList { ::tsox_core::fntrace::enter("new_node_list"); 
        let mut list = crate::ast::node_node_list::NodeList::new(nodes);
        list.loc = TextRange::undefined();
        list
    }

    pub fn new_modifier_list(&self, nodes: Vec<Arc<Node>>) -> ModifierList { ::tsox_core::fntrace::enter("new_modifier_list"); 
        let flags = crate::ast::utilities_modifiers::modifiers_to_flags(&nodes);
        ModifierList::new(nodes, flags)
    }

    pub fn new_modifier(
        &self,
        kind: crate::ast::syntax_kind_generated::SyntaxKind,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_modifier"); 
        self.new_token(kind)
    }

    pub fn new_token(&self, kind: crate::ast::syntax_kind_generated::SyntaxKind) -> Arc<Node> { ::tsox_core::fntrace::enter("new_token"); 
        self.node_count.fetch_add(1, Ordering::Relaxed);
        Arc::new(Node::new(kind, NodeData::Token))
    }

    pub fn new_comment_range(
        &self,
        kind: crate::ast::syntax_kind_generated::SyntaxKind,
        pos: usize,
        end: usize,
        has_trailing_new_line: bool,
    ) -> crate::scanner::CommentRange { ::tsox_core::fntrace::enter("new_comment_range"); 
        use crate::ast::syntax_kind_generated::SyntaxKind;
        let range_kind = if kind == SyntaxKind::MultiLineCommentTrivia {
            crate::scanner::is_jsx_line_break::CommentRangeKind::MultiLine
        } else {
            crate::scanner::is_jsx_line_break::CommentRangeKind::SingleLine
        };
        crate::scanner::CommentRange {
            pos,
            end,
            kind: range_kind,
            has_trailing_new_line,
        }
    }

    pub fn new_source_file(
        &self,
        opts: SourceFileParseOptions,
        text: String,
        statements: Arc<crate::ast::node_node_list::NodeList>,
        end_of_file_token: Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_source_file"); 
        assert!(
            tsox_core::tspath::get_encoded_root_length(&opts.file_name) != 0
                && opts.file_name == tsox_core::tspath::normalize_path(&opts.file_name),
            "fileName should be normalized and absolute: {:?}",
            opts.file_name
        );
        self.node_count.fetch_add(1, Ordering::Relaxed);
        Arc::new(Node::new(
            crate::ast::syntax_kind_generated::SyntaxKind::SourceFile,
            NodeData::SourceFile(crate::ast::node_data_generated::SourceFileData {
                statements,
                end_of_file_token,
                global_exports: None,
            }),
        ))
    }
}

impl Default for NodeFactory {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        Self::new()
    }
}

pub fn get_declaration_map(
    file: &SourceFile,
    symbols: &NodeSymbolMap,
) -> HashMap<String, Vec<Arc<Node>>> { ::tsox_core::fntrace::enter("get_declaration_map"); 
    let mut result: HashMap<String, Vec<Arc<Node>>> = HashMap::new();

    fn add_declaration(
        declaration: &Arc<Node>,
        result: &mut HashMap<String, Vec<Arc<Node>>>,
    ) { ::tsox_core::fntrace::enter("add_declaration"); 
        let name = super::m3b::get_declaration_name(declaration);
        if !name.is_empty() {
            result.entry(name).or_default().push(declaration.clone());
        }
    }

    fn node_body(node: &Node) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("node_body"); 
        match &node.data {
            NodeData::FunctionDeclaration(d) => d.body.clone(),
            NodeData::FunctionExpression(d) => Some(d.body.clone()),
            NodeData::MethodDeclaration(d) => d.body.clone(),
            NodeData::ConstructorDeclaration(d) => d.body.clone(),
            NodeData::GetAccessorDeclaration(d) => d.body.clone(),
            NodeData::SetAccessorDeclaration(d) => d.body.clone(),
            _ => None,
        }
    }

    fn visit(
        node: &Arc<Node>,
        symbols: &NodeSymbolMap,
        result: &mut HashMap<String, Vec<Arc<Node>>>,
    ) -> bool { ::tsox_core::fntrace::enter("visit"); 
        match node.kind {
            crate::ast::syntax_kind_generated::SyntaxKind::FunctionDeclaration
            | crate::ast::syntax_kind_generated::SyntaxKind::FunctionExpression
            | crate::ast::syntax_kind_generated::SyntaxKind::MethodDeclaration
            | crate::ast::syntax_kind_generated::SyntaxKind::MethodSignature => {
                let declaration_name = super::m3b::get_declaration_name(node);
                if !declaration_name.is_empty() {
                    let declarations = result.entry(declaration_name).or_default();
                    let last = declarations.last().cloned();
                    match last {
                        Some(last) => {
                            let same_parent = node
                                .parent()
                                .zip(last.parent())
                                .map(|(np, lp)| Arc::ptr_eq(&np, &lp))
                                .unwrap_or(false);
                            let same_symbol = match (symbols.symbol_of(node), symbols.symbol_of(&last)) {
                                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                                (None, None) => true,
                                _ => false,
                            };
                            if same_parent && same_symbol {
                                if node_body(node).is_some() && node_body(&last).is_none() {
                                    let n = declarations.len();
                                    declarations[n - 1] = node.clone();
                                }
                            } else {
                                declarations.push(node.clone());
                            }
                        }
                        None => declarations.push(node.clone()),
                    }
                }
                for_each_child(node, |child| {
                    visit(child, symbols, result);
                    false
                });
            }
            crate::ast::syntax_kind_generated::SyntaxKind::ClassDeclaration
            | crate::ast::syntax_kind_generated::SyntaxKind::ClassExpression
            | crate::ast::syntax_kind_generated::SyntaxKind::InterfaceDeclaration
            | crate::ast::syntax_kind_generated::SyntaxKind::TypeAliasDeclaration
            | crate::ast::syntax_kind_generated::SyntaxKind::EnumDeclaration
            | crate::ast::syntax_kind_generated::SyntaxKind::ModuleDeclaration
            | crate::ast::syntax_kind_generated::SyntaxKind::ImportEqualsDeclaration
            | crate::ast::syntax_kind_generated::SyntaxKind::ImportClause
            | crate::ast::syntax_kind_generated::SyntaxKind::NamespaceImport
            | crate::ast::syntax_kind_generated::SyntaxKind::GetAccessor
            | crate::ast::syntax_kind_generated::SyntaxKind::SetAccessor
            | crate::ast::syntax_kind_generated::SyntaxKind::TypeLiteral => {
                add_declaration(node, result);
                for_each_child(node, |child| {
                    visit(child, symbols, result);
                    false
                });
            }
            crate::ast::syntax_kind_generated::SyntaxKind::ImportSpecifier
            | crate::ast::syntax_kind_generated::SyntaxKind::ExportSpecifier => {
                if super::m3b::property_name(node).is_some() {
                    add_declaration(node, result);
                }
            }
            crate::ast::syntax_kind_generated::SyntaxKind::Parameter => {
                if node.has_syntactic_modifier(ModifierFlags::ParameterPropertyModifier) {
                    visit_name_holding_declaration(node, symbols, result);
                }
            }
            crate::ast::syntax_kind_generated::SyntaxKind::VariableDeclaration
            | crate::ast::syntax_kind_generated::SyntaxKind::BindingElement => {
                visit_name_holding_declaration(node, symbols, result);
            }
            crate::ast::syntax_kind_generated::SyntaxKind::EnumMember
            | crate::ast::syntax_kind_generated::SyntaxKind::PropertyDeclaration
            | crate::ast::syntax_kind_generated::SyntaxKind::PropertySignature => {
                add_declaration(node, result);
            }
            crate::ast::syntax_kind_generated::SyntaxKind::ExportDeclaration => {
                let export_clause = match &node.data {
                    NodeData::ExportDeclaration(d) => d.export_clause.clone(),
                    _ => None,
                };
                if let Some(export_clause) = export_clause {
                    if is_named_exports(&export_clause) {
                        for element in super::m3b::elements(&export_clause) {
                            visit(&element.clone(), symbols, result);
                        }
                    } else if let Some(name) = export_clause.name() {
                        visit(&name.clone(), symbols, result);
                    }
                }
            }
            crate::ast::syntax_kind_generated::SyntaxKind::ImportDeclaration => {
                let import_clause = match &node.data {
                    NodeData::ImportDeclaration(d) => d.import_clause.clone(),
                    _ => None,
                };
                if let Some(import_clause) = import_clause {
                    if let Some(name) = import_clause.name() {
                        add_declaration(&name.clone(), result);
                    }
                    let named_bindings = match &import_clause.data {
                        NodeData::ImportClause(d) => d.named_bindings.clone(),
                        _ => None,
                    };
                    if let Some(named_bindings) = named_bindings {
                        if named_bindings.kind
                            == crate::ast::syntax_kind_generated::SyntaxKind::NamespaceImport
                        {
                            add_declaration(&named_bindings, result);
                        } else {
                            for element in super::m3b::elements(&named_bindings) {
                                visit(&element.clone(), symbols, result);
                            }
                        }
                    }
                }
            }
            crate::ast::syntax_kind_generated::SyntaxKind::BinaryExpression => {
                if matches!(
                    super::m3e_4::get_assignment_declaration_kind(node),
                    super::m3e_4::JsDeclarationKind::ExportsProperty
                        | super::m3e_4::JsDeclarationKind::ThisProperty
                        | super::m3e_4::JsDeclarationKind::Property
                ) {
                    add_declaration(node, result);
                }
                for_each_child(node, |child| {
                    visit(child, symbols, result);
                    false
                });
            }
            _ => {
                for_each_child(node, |child| {
                    visit(child, symbols, result);
                    false
                });
            }
        }
        false
    }

    fn visit_name_holding_declaration(
        node: &Arc<Node>,
        symbols: &NodeSymbolMap,
        result: &mut HashMap<String, Vec<Arc<Node>>>,
    ) { ::tsox_core::fntrace::enter("visit_name_holding_declaration"); 
        if let Some(name) = node.name() {
            let name = name.clone();
            if is_binding_pattern(&name) {
                for_each_child(&name, |child| {
                    visit(child, symbols, result);
                    false
                });
            } else {
                if let Some(init) = super::m3b::initializer(node) {
                    visit(&init.clone(), symbols, result);
                }
                add_declaration(node, result);
            }
        }
    }

    for_each_child(&file.node, |child| {
        visit(child, symbols, &mut result);
        false
    });
    result
}
