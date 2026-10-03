#![allow(unused_imports)]
use std::sync::Arc;
use std::sync::atomic::Ordering;
use crate::ast::node::{Node, SourceFile};
use crate::ast::node_node_list::NodeList;
use super::m3b_2::NodeFactory;
use crate::ast::node_data_generated::*;
use crate::scanner::error_callback::TokenFlags;
use crate::ast::syntax_kind_generated::SyntaxKind;
use crate::ast::subtree_facts::*;
use crate::ast::visitor::NodeVisitor;
use super::m3d::*;

impl SourceFile {
    pub fn copy_from(&mut self, other: &SourceFile) { ::tsox_core::fntrace::enter("copy_from"); 
        self.language_variant = other.language_variant;
        self.script_kind = other.script_kind;
        self.is_declaration_file = other.is_declaration_file;
        self.uses_uri_style_node_core_modules = other.uses_uri_style_node_core_modules;
        self.imports = other.imports.clone();
        self.module_augmentations = other.module_augmentations.clone();
        self.ambient_module_names = other.ambient_module_names.clone();
        self.comment_directives = other.comment_directives.clone();
        self.common_js_module_indicator = other.common_js_module_indicator.clone();
        self.external_module_indicator = other.external_module_indicator.clone();
        self.referenced_files = other.referenced_files.clone();
        self.type_reference_directives = other.type_reference_directives.clone();
        self.lib_reference_directives = other.lib_reference_directives.clone();
        self.supplemental_source_files = other.supplemental_source_files.clone();
        let node_ptr: *mut Node = match Arc::get_mut(&mut self.node) {
            Some(v) => v as *mut Node,
            None => Arc::as_ptr(&self.node) as *mut Node,
        };
        unsafe {
            (*node_ptr).flags |= other.node.flags;
        }
    }

    pub fn compute_subtree_facts(&self) -> SubtreeFacts { ::tsox_core::fntrace::enter("compute_subtree_facts"); 
        match &self.node.data {
            NodeData::SourceFile(d) => propagate_node_list_subtree_facts(
                Some(&d.statements),
                |n: &Arc<Node>| propagate_subtree_facts(Some(n)),
            ),
            _ => SUBTREE_FACTS_NONE,
        }
    }
}

pub struct SourceFileDataCell<T> {
    once: std::sync::Once,
    value: std::sync::Mutex<Option<T>>,
}

impl<T> Default for SourceFileDataCell<T> {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        Self {
            once: std::sync::Once::new(),
            value: std::sync::Mutex::new(None),
        }
    }
}

impl<T: Clone> SourceFileDataCell<T> {
    pub fn get_or_init(&self, compute: impl FnOnce() -> T) -> T { ::tsox_core::fntrace::enter("get_or_init"); 
        self.once.call_once(|| {
            *self.value.lock().unwrap() = Some(compute());
        });
        self.value.lock().unwrap().clone().unwrap()
    }
}

pub struct SourceFileDataKey<T> {
    key: u64,
    _marker: std::marker::PhantomData<T>,
}

static SOURCE_FILE_DATA_KEY_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl<T> SourceFileDataKey<T> {
    pub fn new() -> Self { ::tsox_core::fntrace::enter("new"); 
        Self {
            key: SOURCE_FILE_DATA_KEY_COUNTER.fetch_add(1, Ordering::Relaxed) + 1,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> Default for SourceFileDataKey<T> {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        Self::new()
    }
}

type SourceFileDataCells = std::collections::HashMap<
    (u64, u64),
    Arc<dyn std::any::Any + Send + Sync>,
>;

fn global_source_file_data_cells() -> &'static std::sync::Mutex<SourceFileDataCells> { ::tsox_core::fntrace::enter("global_source_file_data_cells"); 
    static CELLS: std::sync::OnceLock<std::sync::Mutex<SourceFileDataCells>> =
        std::sync::OnceLock::new();
    CELLS.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

pub fn get_source_file_data_cell<T: Send + Sync + 'static>(
    file: &SourceFile,
    key: &SourceFileDataKey<T>,
) -> Arc<SourceFileDataCell<T>> { ::tsox_core::fntrace::enter("get_source_file_data_cell"); 
    if key.key == 0 {
        panic!("invalid SourceFileDataKey; use NewSourceFileDataKey");
    }
    let mut cells = global_source_file_data_cells().lock().unwrap();
    let cell = cells
        .entry((file.id(), key.key))
        .or_insert_with(|| Arc::new(SourceFileDataCell::<T>::default()))
        .clone();
    drop(cells);
    cell.downcast::<SourceFileDataCell<T>>()
        .expect("SourceFileDataCell type mismatch")
}

fn file_token_factory(file: &SourceFile) -> Arc<NodeFactory> { ::tsox_core::fntrace::enter("file_token_factory"); 
    static TOKEN_FACTORY_KEY: std::sync::OnceLock<SourceFileDataKey<Arc<NodeFactory>>> =
        std::sync::OnceLock::new();
    let key = TOKEN_FACTORY_KEY.get_or_init(|| SourceFileDataKey::<Arc<NodeFactory>>::new());
    get_source_file_data_cell(file, key).get_or_init(|| Arc::new(NodeFactory::new()))
}

pub fn create_token(
    kind: SyntaxKind,
    file: &SourceFile,
    pos: usize,
    end: usize,
    flags: TokenFlags,
) -> Arc<Node> { ::tsox_core::fntrace::enter("create_token"); 
    let token_factory = file_token_factory(file);
    let text = &file.text[pos..end];
    match kind {
        SyntaxKind::NumericLiteral => token_factory.new_numeric_literal(text, flags),
        SyntaxKind::BigIntLiteral => token_factory.new_big_int_literal(text, flags),
        SyntaxKind::StringLiteral => token_factory.new_string_literal(text, flags),
        SyntaxKind::JsxText | SyntaxKind::JsxTextAllWhiteSpaces => {
            token_factory.new_jsx_text(text, kind == SyntaxKind::JsxTextAllWhiteSpaces)
        }
        SyntaxKind::RegularExpressionLiteral => token_factory.new_regular_expression_literal(text, flags),
        SyntaxKind::NoSubstitutionTemplateLiteral => {
            token_factory.new_no_substitution_template_literal(text, flags)
        }
        SyntaxKind::TemplateHead => token_factory.new_template_head(text, "", flags),
        SyntaxKind::TemplateMiddle => token_factory.new_template_middle(text, "", flags),
        SyntaxKind::TemplateTail => token_factory.new_template_tail(text, "", flags),
        SyntaxKind::Identifier => token_factory.new_identifier(text),
        SyntaxKind::PrivateIdentifier => token_factory.new_private_identifier(text),
        _ => token_factory.new_token(kind),
    }
}

impl NodeFactory {
    pub fn new_numeric_literal(&self, text: &str, token_flags: TokenFlags) -> Arc<Node> { ::tsox_core::fntrace::enter("new_numeric_literal"); 
        Arc::new(Node::new(
            SyntaxKind::NumericLiteral,
            NodeData::NumericLiteral(NumericLiteralData {
                text: text.to_string(),
                token_flags,
            }),
        ))
    }

    pub fn new_string_literal(&self, text: &str, token_flags: TokenFlags) -> Arc<Node> { ::tsox_core::fntrace::enter("new_string_literal"); 
        Arc::new(Node::new(
            SyntaxKind::StringLiteral,
            NodeData::StringLiteral(StringLiteralData {
                text: text.to_string(),
                token_flags,
            }),
        ))
    }

    pub fn new_identifier(&self, text: &str) -> Arc<Node> { ::tsox_core::fntrace::enter("new_identifier"); 
        Arc::new(Node::new(
            SyntaxKind::Identifier,
            NodeData::Identifier(IdentifierData {
                text: text.to_string(),
            }),
        ))
    }

    pub fn new_big_int_literal(&self, text: &str, token_flags: TokenFlags) -> Arc<Node> { ::tsox_core::fntrace::enter("new_big_int_literal"); 
        Arc::new(Node::new(
            SyntaxKind::BigIntLiteral,
            NodeData::BigIntLiteral(BigIntLiteralData {
                text: text.to_string(),
                token_flags,
            }),
        ))
    }

    pub fn new_jsx_text(&self, text: &str, contains_only_trivia_white_spaces: bool) -> Arc<Node> { ::tsox_core::fntrace::enter("new_jsx_text"); 
        Arc::new(Node::new(
            SyntaxKind::JsxText,
            NodeData::JsxText(JsxTextData {
                text: text.to_string(),
                contains_only_trivia_white_spaces,
            }),
        ))
    }

    pub fn new_regular_expression_literal(
        &self,
        text: &str,
        token_flags: TokenFlags,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_regular_expression_literal"); 
        Arc::new(Node::new(
            SyntaxKind::RegularExpressionLiteral,
            NodeData::RegularExpressionLiteral(RegularExpressionLiteralData {
                text: text.to_string(),
                token_flags,
            }),
        ))
    }

    pub fn new_no_substitution_template_literal(
        &self,
        text: &str,
        template_flags: TokenFlags,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_no_substitution_template_literal"); 
        Arc::new(Node::new(
            SyntaxKind::NoSubstitutionTemplateLiteral,
            NodeData::NoSubstitutionTemplateLiteral(NoSubstitutionTemplateLiteralData {
                text: text.to_string(),
                template_flags,
            }),
        ))
    }

    pub fn new_template_head(&self, text: &str, raw_text: &str, template_flags: TokenFlags) -> Arc<Node> { ::tsox_core::fntrace::enter("new_template_head"); 
        Arc::new(Node::new(
            SyntaxKind::TemplateHead,
            NodeData::TemplateHead(TemplateHeadData {
                text: text.to_string(),
                raw_text: raw_text.to_string(),
                template_flags,
            }),
        ))
    }

    pub fn new_template_middle(&self, text: &str, raw_text: &str, template_flags: TokenFlags) -> Arc<Node> { ::tsox_core::fntrace::enter("new_template_middle"); 
        Arc::new(Node::new(
            SyntaxKind::TemplateMiddle,
            NodeData::TemplateMiddle(TemplateMiddleData {
                text: text.to_string(),
                raw_text: raw_text.to_string(),
                template_flags,
            }),
        ))
    }

    pub fn new_template_tail(&self, text: &str, raw_text: &str, template_flags: TokenFlags) -> Arc<Node> { ::tsox_core::fntrace::enter("new_template_tail"); 
        Arc::new(Node::new(
            SyntaxKind::TemplateTail,
            NodeData::TemplateTail(TemplateTailData {
                text: text.to_string(),
                raw_text: raw_text.to_string(),
                template_flags,
            }),
        ))
    }

    pub fn new_private_identifier(&self, text: &str) -> Arc<Node> { ::tsox_core::fntrace::enter("new_private_identifier"); 
        Arc::new(Node::new(
            SyntaxKind::PrivateIdentifier,
            NodeData::PrivateIdentifier(PrivateIdentifierData {
                text: text.to_string(),
            }),
        ))
    }

    pub fn update_jsdoc_parameter_or_property_tag(
        &self,
        kind: SyntaxKind,
        node: &JSDocParameterOrPropertyTagData,
        tag_name: Arc<Node>,
        name: Arc<Node>,
        is_bracketed: bool,
        type_expression: Option<Arc<Node>>,
        is_name_first: bool,
        comment: Option<Arc<NodeList>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_jsdoc_parameter_or_property_tag"); 
        let _ = self;
        Arc::new(Node::new(
            kind,
            NodeData::JSDocParameterOrPropertyTag(JSDocParameterOrPropertyTagData {
                tag_name,
                name,
                is_bracketed,
                type_expression: type_expression.or_else(|| node.type_expression.clone()),
                is_name_first,
                comment: comment.or_else(|| node.comment.clone()),
            }),
        ))
    }
}

pub fn for_each_child_jsdoc_parameter_or_property_tag(
    node: &JSDocParameterOrPropertyTagData,
    v: &mut dyn FnMut(&Node) -> bool,
) -> bool { ::tsox_core::fntrace::enter("for_each_child_jsdoc_parameter_or_property_tag"); 
    visit(v, Some(node.tag_name.as_ref()))
        || (node.is_name_first
            && (visit(v, Some(node.name.as_ref())) || visit(v, node.type_expression.as_deref())))
        || (!node.is_name_first
            && (visit(v, node.type_expression.as_deref()) || visit(v, Some(node.name.as_ref()))))
        || visit_node_list(v, node.comment.as_deref())
}

pub fn visit_each_child_jsdoc_parameter_or_property_tag(
    node: &Arc<Node>,
    v: &mut NodeVisitor,
) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_each_child_jsdoc_parameter_or_property_tag"); 
    let NodeData::JSDocParameterOrPropertyTag(data) = &node.data else {
        return Arc::clone(node);
    };
    let comment = v.visit_node_list(data.comment.as_deref());
    let comment = (!comment.is_empty() || data.comment.is_some())
        .then(|| Arc::new(crate::ast::node_node_list::NodeList::new(comment)));
    let tag_name = v.visit_node(&data.tag_name);
    let name = v.visit_node(&data.name);
    let type_expression = v.visit_node_opt(data.type_expression.as_ref());
    v.factory.update_jsdoc_parameter_or_property_tag(
        node.kind,
        data,
        tag_name,
        name,
        data.is_bracketed,
        type_expression,
        data.is_name_first,
        comment,
    )
}
