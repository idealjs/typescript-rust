#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{self as ndg, NodeData};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::{ModifierList, NodeList, SourceFile};

use crate::printer::NodeFactory;

pub trait R38K9SourceFileNodeExt {
    fn as_node(&self) -> Arc<Node>;
}

impl R38K9SourceFileNodeExt for SourceFile {
    fn as_node(&self) -> Arc<Node> { ::tsox_core::fntrace::enter("as_node"); 
        self.node.clone()
    }
}

impl R38K9SourceFileNodeExt for Arc<SourceFile> {
    fn as_node(&self) -> Arc<Node> { ::tsox_core::fntrace::enter("as_node"); 
        self.node.clone()
    }
}

pub trait R38K9NodeCastExt {
    fn as_binary_expression(&self) -> &ndg::BinaryExpressionData;
    fn as_catch_clause(&self) -> &ndg::CatchClauseData;
    fn as_tagged_template_expression(&self) -> &ndg::TaggedTemplateExpressionData;
    fn as_template_expression(&self) -> &ndg::TemplateExpressionData;
    fn as_template_span(&self) -> &ndg::TemplateSpanData;
    fn as_binding_element(&self) -> &ndg::BindingElementData;
    fn as_binding_pattern(&self) -> &ndg::BindingPatternData;
    fn as_variable_declaration(&self) -> &ndg::VariableDeclarationData;
    fn as_shorthand_property_assignment(&self) -> &ndg::ShorthandPropertyAssignmentData;
    fn as_for_statement(&self) -> &ndg::ForStatementData;
    fn as_import_equals_declaration(&self) -> &ndg::ImportEqualsDeclarationData;
    fn as_conditional_expression(&self) -> &ndg::ConditionalExpressionData;
    fn as_import_attribute(&self) -> &ndg::ImportAttributeData;
    fn as_block_data(&self) -> &ndg::BlockData;
    fn as_source_file_data(&self) -> &ndg::SourceFileData;
    fn initializer(&self) -> Option<&Arc<Node>>;
    fn body(&self) -> Option<&Arc<Node>>;
    fn call_arguments(&self) -> &[Arc<Node>];
    fn tag_name(&self) -> Arc<Node>;
    fn statement_list(&self) -> Arc<NodeList>;
    fn block_statements(&self) -> Arc<NodeList>;
    fn template_literal_like_data(&self) -> crate::mig::m4i::TemplateLiteralLikeDataBase;
}

macro_rules! cast_data {
    ($self:expr, $variant:ident) => {
        match &$self.data {
            NodeData::$variant(d) => d,
            _ => panic!(concat!("unexpected node for ", stringify!($variant), ": {:?}"), $self.kind),
        }
    };
}

impl R38K9NodeCastExt for Arc<Node> {
    fn as_binary_expression(&self) -> &ndg::BinaryExpressionData { ::tsox_core::fntrace::enter("as_binary_expression"); 
        cast_data!(self, BinaryExpression)
    }

    fn as_catch_clause(&self) -> &ndg::CatchClauseData { ::tsox_core::fntrace::enter("as_catch_clause"); 
        cast_data!(self, CatchClause)
    }

    fn as_tagged_template_expression(&self) -> &ndg::TaggedTemplateExpressionData { ::tsox_core::fntrace::enter("as_tagged_template_expression"); 
        cast_data!(self, TaggedTemplateExpression)
    }

    fn as_template_expression(&self) -> &ndg::TemplateExpressionData { ::tsox_core::fntrace::enter("as_template_expression"); 
        cast_data!(self, TemplateExpression)
    }

    fn as_template_span(&self) -> &ndg::TemplateSpanData { ::tsox_core::fntrace::enter("as_template_span"); 
        cast_data!(self, TemplateSpan)
    }

    fn as_binding_element(&self) -> &ndg::BindingElementData { ::tsox_core::fntrace::enter("as_binding_element"); 
        cast_data!(self, BindingElement)
    }

    fn as_binding_pattern(&self) -> &ndg::BindingPatternData { ::tsox_core::fntrace::enter("as_binding_pattern"); 
        cast_data!(self, BindingPattern)
    }

    fn as_variable_declaration(&self) -> &ndg::VariableDeclarationData { ::tsox_core::fntrace::enter("as_variable_declaration"); 
        cast_data!(self, VariableDeclaration)
    }

    fn as_shorthand_property_assignment(&self) -> &ndg::ShorthandPropertyAssignmentData { ::tsox_core::fntrace::enter("as_shorthand_property_assignment"); 
        cast_data!(self, ShorthandPropertyAssignment)
    }

    fn as_for_statement(&self) -> &ndg::ForStatementData { ::tsox_core::fntrace::enter("as_for_statement"); 
        cast_data!(self, ForStatement)
    }

    fn as_import_equals_declaration(&self) -> &ndg::ImportEqualsDeclarationData { ::tsox_core::fntrace::enter("as_import_equals_declaration"); 
        cast_data!(self, ImportEqualsDeclaration)
    }

    fn as_conditional_expression(&self) -> &ndg::ConditionalExpressionData { ::tsox_core::fntrace::enter("as_conditional_expression"); 
        cast_data!(self, ConditionalExpression)
    }

    fn as_import_attribute(&self) -> &ndg::ImportAttributeData { ::tsox_core::fntrace::enter("as_import_attribute"); 
        cast_data!(self, ImportAttribute)
    }

    fn as_block_data(&self) -> &ndg::BlockData { ::tsox_core::fntrace::enter("as_block_data"); 
        cast_data!(self, Block)
    }

    fn as_source_file_data(&self) -> &ndg::SourceFileData { ::tsox_core::fntrace::enter("as_source_file_data"); 
        cast_data!(self, SourceFile)
    }

    fn initializer(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("initializer"); 
        match &self.data {
            NodeData::VariableDeclaration(d) => d.initializer.as_ref(),
            NodeData::ParameterDeclaration(d) => d.initializer.as_ref(),
            NodeData::BindingElement(d) => d.initializer.as_ref(),
            NodeData::PropertyDeclaration(d) => d.initializer.as_ref(),
            NodeData::PropertySignatureDeclaration(d) => Some(&d.initializer),
            NodeData::PropertyAssignment(d) => Some(&d.initializer),
            NodeData::ShorthandPropertyAssignment(d) => d.object_assignment_initializer.as_ref(),
            NodeData::EnumMember(d) => d.initializer.as_ref(),
            NodeData::JsxAttribute(d) => d.initializer.as_ref(),
            _ => None,
        }
    }

    fn body(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("body"); 
        match &self.data {
            NodeData::ArrowFunction(d) => Some(&d.body),
            NodeData::FunctionExpression(d) => Some(&d.body),
            NodeData::MethodDeclaration(d) => d.body.as_ref(),
            NodeData::GetAccessorDeclaration(d) => d.body.as_ref(),
            NodeData::SetAccessorDeclaration(d) => d.body.as_ref(),
            NodeData::ConstructorDeclaration(d) => d.body.as_ref(),
            NodeData::FunctionDeclaration(d) => d.body.as_ref(),
            _ => None,
        }
    }

    fn call_arguments(&self) -> &[Arc<Node>] { ::tsox_core::fntrace::enter("call_arguments"); 
        match &self.data {
            NodeData::CallExpression(d) => &d.arguments.nodes,
            NodeData::NewExpression(d) => match &d.arguments {
                Some(list) => &list.nodes,
                None => &[] as &[Arc<Node>],
            },
            _ => &[],
        }
    }

    fn tag_name(&self) -> Arc<Node> { ::tsox_core::fntrace::enter("tag_name"); 
        match &self.data {
            NodeData::JsxOpeningElement(d) => d.tag_name.clone(),
            NodeData::JsxClosingElement(d) => d.tag_name.clone(),
            NodeData::JsxSelfClosingElement(d) => d.tag_name.clone(),
            _ => panic!("tag_name() on {:?}", self.kind),
        }
    }

    fn statement_list(&self) -> Arc<NodeList> { ::tsox_core::fntrace::enter("statement_list"); 
        match &self.data {
            NodeData::Block(d) => d.statements.clone(),
            NodeData::SourceFile(d) => d.statements.clone(),
            NodeData::ModuleBlock(d) => d.statements.clone(),
            _ => panic!("statement_list() on {:?}", self.kind),
        }
    }

    fn block_statements(&self) -> Arc<NodeList> { ::tsox_core::fntrace::enter("block_statements"); 
        self.statement_list()
    }

    fn template_literal_like_data(&self) -> crate::mig::m4i::TemplateLiteralLikeDataBase { ::tsox_core::fntrace::enter("template_literal_like_data"); 
        match &self.data {
            NodeData::NoSubstitutionTemplateLiteral(d) => crate::mig::m4i::TemplateLiteralLikeDataBase {
                text: d.text.clone(),
                raw_text: d.text.clone(),
                template_flags: d.template_flags,
            },
            NodeData::TemplateHead(d) => crate::mig::m4i::TemplateLiteralLikeDataBase {
                text: d.text.clone(),
                raw_text: d.raw_text.clone(),
                template_flags: d.template_flags,
            },
            NodeData::TemplateMiddle(d) => crate::mig::m4i::TemplateLiteralLikeDataBase {
                text: d.text.clone(),
                raw_text: d.raw_text.clone(),
                template_flags: d.template_flags,
            },
            NodeData::TemplateTail(d) => crate::mig::m4i::TemplateLiteralLikeDataBase {
                text: d.text.clone(),
                raw_text: d.raw_text.clone(),
                template_flags: d.template_flags,
            },
            _ => panic!("template_literal_like_data() on {:?}", self.kind),
        }
    }
}

pub trait R38K9ParamDeclDataExt {
    fn name(&self) -> &Arc<Node>;
    fn dot_dot_dot_token(&self) -> Option<&Arc<Node>>;
    fn initializer(&self) -> Option<&Arc<Node>>;
}

impl R38K9ParamDeclDataExt for ndg::ParameterDeclarationData {
    fn name(&self) -> &Arc<Node> { ::tsox_core::fntrace::enter("name"); 
        &self.name
    }
    fn dot_dot_dot_token(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("dot_dot_dot_token"); 
        self.dot_dot_dot_token.as_ref()
    }
    fn initializer(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("initializer"); 
        self.initializer.as_ref()
    }
}

pub trait R38K9MethodDeclDataExt {
    fn asterisk_token(&self) -> Option<&Arc<Node>>;
    fn postfix_token(&self) -> Option<&Arc<Node>>;
}

impl R38K9MethodDeclDataExt for ndg::MethodDeclarationData {
    fn asterisk_token(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("asterisk_token"); 
        self.asterisk_token.as_ref()
    }
    fn postfix_token(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("postfix_token"); 
        self.postfix_token.as_ref()
    }
}

pub trait R38K9FunctionDeclDataExt {
    fn asterisk_token(&self) -> Option<&Arc<Node>>;
}

impl R38K9FunctionDeclDataExt for ndg::FunctionDeclarationData {
    fn asterisk_token(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("asterisk_token"); 
        self.asterisk_token.as_ref()
    }
}

pub trait R38K9ArrowFnDataExt {
    fn equals_greater_than_token(&self) -> &Arc<Node>;
}

impl R38K9ArrowFnDataExt for ndg::ArrowFunctionData {
    fn equals_greater_than_token(&self) -> &Arc<Node> { ::tsox_core::fntrace::enter("equals_greater_than_token"); 
        &self.equals_greater_than_token
    }
}

pub trait R38K9FnExprDataExt {
    fn asterisk_token(&self) -> Option<&Arc<Node>>;
}

impl R38K9FnExprDataExt for ndg::FunctionExpressionData {
    fn asterisk_token(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("asterisk_token"); 
        self.asterisk_token.as_ref()
    }
}

impl<'a> NodeFactory<'a> {
    pub fn r38k9_new_spread_element(&self, expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("r38k9_new_spread_element"); 
        Arc::new(Node::new(
            SyntaxKind::SpreadElement,
            NodeData::SpreadElement(ndg::SpreadElementData {
                expression: expression.clone(),
            }),
        ))
    }

    pub fn r38k9_new_spread_assignment(&self, expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("r38k9_new_spread_assignment"); 
        Arc::new(Node::new(
            SyntaxKind::SpreadAssignment,
            NodeData::SpreadAssignment(ndg::SpreadAssignmentData {
                expression: expression.clone(),
            }),
        ))
    }

    pub fn r38k9_new_logical_or_expression(&self, left: &Arc<Node>, right: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("r38k9_new_logical_or_expression"); 
        self.new_binary_expression(
            None,
            left,
            None,
            &self.new_token(SyntaxKind::BarBarToken),
            right,
        )
    }

    pub fn r38k9_new_template_object_helper(
        &self,
        cooked: &Arc<Node>,
        raw: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("r38k9_new_template_object_helper"); 
        self.new_call_expression(
            &self.new_identifier("__makeTemplateObject"),
            None,
            None,
            self.new_node_list(vec![cooked.clone(), raw.clone()]),
            tsox_frontend::ast::NodeFlags::empty(),
        )
    }
}

pub fn r38k9_same_node(a: &Arc<Node>, b: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("r38k9_same_node"); 
    Arc::ptr_eq(a, b)
}
