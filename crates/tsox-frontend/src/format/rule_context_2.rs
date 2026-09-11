//! Go format/rulecontext.go 的谓词全集（对齐上游命名，snake_case）。

use std::sync::Arc;

use crate::ast::node::Node;
use crate::ast::SyntaxKind;
use crate::format::scanner::TextRangeWithKind;
use tsox_core::core::text::TextRange;
use crate::format::rule_context::{FormattingContext, FormatRequestKind, Tristate};
use crate::format::{FormatCodeSettings, SemicolonPreference};

// ---- 选项选择器（fn 指针友好：每个 (选项, 变体) 一个具名谓词） ----

macro_rules! tristate_option_predicates {
    ($field:ident, enabled: $enabled:ident, disabled_or_undef: $disabled:ident) => {
        pub(crate) fn $enabled(context: &mut FormattingContext) -> bool {
            context.options.$field == Tristate::True
        }
        pub(crate) fn $disabled(context: &mut FormattingContext) -> bool {
            context.options.$field != Tristate::True
        }
    };
}

tristate_option_predicates!(insert_space_after_comma_delimiter,
    enabled: is_insert_space_after_comma_enabled,
    disabled_or_undef: is_insert_space_after_comma_disabled_or_undef);
tristate_option_predicates!(insert_space_after_semicolon_in_for_statements,
    enabled: is_insert_space_after_semicolon_in_for_enabled,
    disabled_or_undef: is_insert_space_after_semicolon_in_for_disabled_or_undef);
tristate_option_predicates!(insert_space_after_constructor,
    enabled: is_insert_space_after_constructor_enabled,
    disabled_or_undef: is_insert_space_after_constructor_disabled_or_undef);
tristate_option_predicates!(insert_space_after_keywords_in_control_flow_statements,
    enabled: is_insert_space_after_keywords_in_control_enabled,
    disabled_or_undef: is_insert_space_after_keywords_in_control_disabled_or_undef);
tristate_option_predicates!(insert_space_after_function_keyword_for_anonymous_functions,
    enabled: is_insert_space_after_anonymous_function_enabled,
    disabled_or_undef: is_insert_space_after_anonymous_function_disabled_or_undef);
tristate_option_predicates!(insert_space_after_opening_and_before_closing_nonempty_parenthesis,
    enabled: is_insert_space_nonempty_paren_enabled,
    disabled_or_undef: is_insert_space_nonempty_paren_disabled_or_undef);
tristate_option_predicates!(insert_space_after_opening_and_before_closing_nonempty_brackets,
    enabled: is_insert_space_nonempty_bracket_enabled,
    disabled_or_undef: is_insert_space_nonempty_bracket_disabled_or_undef);
tristate_option_predicates!(insert_space_after_type_assertion,
    enabled: is_insert_space_after_type_assertion_enabled,
    disabled_or_undef: is_insert_space_after_type_assertion_disabled_or_undef);
tristate_option_predicates!(insert_space_before_function_parenthesis,
    enabled: is_insert_space_before_function_paren_enabled,
    disabled_or_undef: is_insert_space_before_function_paren_disabled_or_undef);

pub(crate) fn is_insert_space_nonempty_braces_enabled_or_undef(context: &mut FormattingContext) -> bool {
    context.options.insert_space_after_opening_and_before_closing_nonempty_braces == Tristate::True
        || context.options.insert_space_after_opening_and_before_closing_nonempty_braces == Tristate::Unknown
}

pub(crate) fn is_insert_space_nonempty_braces_disabled(context: &mut FormattingContext) -> bool {
    context.options.insert_space_after_opening_and_before_closing_nonempty_braces == Tristate::False
}

pub(crate) fn is_insert_space_empty_braces_enabled(context: &mut FormattingContext) -> bool {
    context.options.insert_space_after_opening_and_before_closing_empty_braces == Tristate::True
}

pub(crate) fn is_insert_space_empty_braces_disabled(context: &mut FormattingContext) -> bool {
    context.options.insert_space_after_opening_and_before_closing_empty_braces == Tristate::False
}

pub(crate) fn is_insert_space_template_braces_enabled(context: &mut FormattingContext) -> bool {
    context.options.insert_space_after_opening_and_before_closing_template_string_braces == Tristate::True
}

pub(crate) fn is_insert_space_template_braces_disabled_or_undef(context: &mut FormattingContext) -> bool {
    context.options.insert_space_after_opening_and_before_closing_template_string_braces != Tristate::True
}

pub(crate) fn is_insert_space_jsx_braces_enabled(context: &mut FormattingContext) -> bool {
    context.options.insert_space_after_opening_and_before_closing_jsx_expression_braces == Tristate::True
}

pub(crate) fn is_insert_space_jsx_braces_disabled_or_undef(context: &mut FormattingContext) -> bool {
    context.options.insert_space_after_opening_and_before_closing_jsx_expression_braces != Tristate::True
}

pub(crate) fn is_insert_space_binary_operators_enabled(context: &mut FormattingContext) -> bool {
    context.options.insert_space_before_and_after_binary_operators
}

pub(crate) fn is_insert_space_binary_operators_disabled(context: &mut FormattingContext) -> bool {
    !context.options.insert_space_before_and_after_binary_operators
}

pub(crate) fn is_place_open_brace_newline_functions_enabled(context: &mut FormattingContext) -> bool {
    context.options.place_open_brace_on_new_line_for_functions == Tristate::True
}

pub(crate) fn is_place_open_brace_newline_functions_disabled_or_same_line(context: &mut FormattingContext) -> bool {
    context.options.place_open_brace_on_new_line_for_functions != Tristate::True
        || context.tokens_are_on_same_line()
}

pub(crate) fn is_place_open_brace_newline_control_enabled(context: &mut FormattingContext) -> bool {
    context.options.place_open_brace_on_new_line_for_control_blocks == Tristate::True
}

pub(crate) fn is_place_open_brace_newline_control_disabled_or_same_line(context: &mut FormattingContext) -> bool {
    context.options.place_open_brace_on_new_line_for_control_blocks != Tristate::True
        || context.tokens_are_on_same_line()
}

pub(crate) fn is_semicolon_preference_remove(context: &mut FormattingContext) -> bool {
    context.options.semicolons == SemicolonPreference::Remove
}

pub(crate) fn is_semicolon_preference_insert(context: &mut FormattingContext) -> bool {
    context.options.semicolons == SemicolonPreference::Insert
}

// ---- 上下文谓词 ----

impl FormattingContext {
    pub(crate) fn is_for_context(&mut self) -> bool {
        self.context_node_kind() == SyntaxKind::ForStatement
    }

    pub(crate) fn is_not_for_context(&mut self) -> bool {
        !self.is_for_context()
    }

    pub(crate) fn is_binary_op_context(&mut self) -> bool {
        let Some(node) = &self.context_node else { return false };
        match node.kind {
            SyntaxKind::BinaryExpression => {
                if let crate::ast::NodeData::BinaryExpression(d) = &node.data {
                    d.operator_token.kind != SyntaxKind::CommaToken
                } else {
                    false
                }
            }
            SyntaxKind::ConditionalExpression
            | SyntaxKind::ConditionalType
            | SyntaxKind::AsExpression
            | SyntaxKind::ExportSpecifier
            | SyntaxKind::ImportSpecifier
            | SyntaxKind::TypePredicate
            | SyntaxKind::UnionType
            | SyntaxKind::IntersectionType
            | SyntaxKind::SatisfiesExpression => true,
            SyntaxKind::BindingElement
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ExportAssignment
            | SyntaxKind::VariableDeclaration
            | SyntaxKind::Parameter
            | SyntaxKind::EnumMember
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature => {
                self.current_token_span.kind == SyntaxKind::EqualsToken
                    || self.next_token_span.kind == SyntaxKind::EqualsToken
            }
            SyntaxKind::ForInStatement | SyntaxKind::TypeParameter => {
                self.current_token_span.kind == SyntaxKind::InKeyword
                    || self.next_token_span.kind == SyntaxKind::InKeyword
                    || self.current_token_span.kind == SyntaxKind::EqualsToken
                    || self.next_token_span.kind == SyntaxKind::EqualsToken
            }
            SyntaxKind::ForOfStatement => {
                self.current_token_span.kind == SyntaxKind::OfKeyword
                    || self.next_token_span.kind == SyntaxKind::OfKeyword
            }
            _ => false,
        }
    }

    pub(crate) fn is_not_binary_op_context(&mut self) -> bool {
        !self.is_binary_op_context()
    }

    pub(crate) fn is_type_annotation_context(&mut self) -> bool {
        matches!(
            self.context_node_kind(),
            SyntaxKind::PropertyDeclaration
                | SyntaxKind::PropertySignature
                | SyntaxKind::Parameter
                | SyntaxKind::VariableDeclaration
        ) || crate::ast::utilities_functions::is_function_like_kind(self.context_node_kind())
    }

    pub(crate) fn is_not_type_annotation_context(&mut self) -> bool {
        !self.is_type_annotation_context()
    }

    pub(crate) fn is_conditional_operator_context(&mut self) -> bool {
        matches!(
            self.context_node_kind(),
            SyntaxKind::ConditionalExpression | SyntaxKind::ConditionalType
        )
    }

    pub(crate) fn is_same_line_token_or_before_block_context(&mut self) -> bool {
        self.tokens_are_on_same_line() || self.is_before_block_context()
    }

    pub(crate) fn is_brace_wrapped_context(&mut self) -> bool {
        matches!(
            self.context_node_kind(),
            SyntaxKind::ObjectBindingPattern | SyntaxKind::MappedType
        ) || self.is_single_line_block_context()
    }

    pub(crate) fn is_before_multiline_block_context(&mut self) -> bool {
        self.is_before_block_context()
            && !(self.next_node_all_on_same_line() || self.next_node_block_is_on_one_line())
    }

    pub(crate) fn is_multiline_block_context(&mut self) -> bool {
        self.is_block_context()
            && !(self.context_node_all_on_same_line() || self.context_node_block_is_on_one_line())
    }

    pub(crate) fn is_single_line_block_context(&mut self) -> bool {
        self.is_block_context()
            && (self.context_node_all_on_same_line() || self.context_node_block_is_on_one_line())
    }

    pub(crate) fn is_block_context(&mut self) -> bool {
        self.context_node.as_ref().is_some_and(node_is_block_context)
    }

    pub(crate) fn is_before_block_context(&mut self) -> bool {
        self.next_token_parent
            .as_ref()
            .is_some_and(node_is_block_context)
    }

    pub(crate) fn is_function_decl_context(&mut self) -> bool {
        matches!(
            self.context_node_kind(),
            SyntaxKind::FunctionDeclaration
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::MethodSignature
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor
                | SyntaxKind::CallSignature
                | SyntaxKind::FunctionExpression
                | SyntaxKind::Constructor
                | SyntaxKind::ArrowFunction
                | SyntaxKind::InterfaceDeclaration
        )
    }

    pub(crate) fn is_not_function_decl_context(&mut self) -> bool {
        !self.is_function_decl_context()
    }

    pub(crate) fn is_function_declaration_or_function_expression_context(&mut self) -> bool {
        matches!(
            self.context_node_kind(),
            SyntaxKind::FunctionDeclaration | SyntaxKind::FunctionExpression
        )
    }

    pub(crate) fn is_type_script_decl_with_block_context(&mut self) -> bool {
        self.context_node.as_ref().is_some_and(node_is_type_script_decl_with_block)
    }

    pub(crate) fn is_after_code_block_context(&mut self) -> bool {
        let Some(parent) = &self.current_token_parent else { return false };
        match parent.kind {
            SyntaxKind::ClassDeclaration
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::CatchClause
            | SyntaxKind::ModuleBlock
            | SyntaxKind::SwitchStatement => true,
            SyntaxKind::Block => match parent.parent.as_ref() {
                None => true,
                Some(gp) => !matches!(gp.kind, SyntaxKind::ArrowFunction | SyntaxKind::FunctionExpression),
            },
            _ => false,
        }
    }

    pub(crate) fn is_control_decl_context(&mut self) -> bool {
        matches!(
            self.context_node_kind(),
            SyntaxKind::IfStatement
                | SyntaxKind::SwitchStatement
                | SyntaxKind::ForStatement
                | SyntaxKind::ForInStatement
                | SyntaxKind::ForOfStatement
                | SyntaxKind::WhileStatement
                | SyntaxKind::TryStatement
                | SyntaxKind::DoStatement
                | SyntaxKind::WithStatement
                | SyntaxKind::CatchClause
        )
    }

    pub(crate) fn is_object_context(&mut self) -> bool {
        self.context_node_kind() == SyntaxKind::ObjectLiteralExpression
    }

    pub(crate) fn is_function_call_or_new_context(&mut self) -> bool {
        matches!(
            self.context_node_kind(),
            SyntaxKind::CallExpression | SyntaxKind::NewExpression
        )
    }

    pub(crate) fn is_previous_token_not_comma(&mut self) -> bool {
        self.current_token_span.kind != SyntaxKind::CommaToken
    }

    pub(crate) fn is_next_token_not_close_bracket(&mut self) -> bool {
        self.next_token_span.kind != SyntaxKind::CloseBracketToken
    }

    pub(crate) fn is_next_token_not_close_paren(&mut self) -> bool {
        self.next_token_span.kind != SyntaxKind::CloseParenToken
    }

    pub(crate) fn is_arrow_function_context(&mut self) -> bool {
        self.context_node_kind() == SyntaxKind::ArrowFunction
    }

    pub(crate) fn is_import_type_context(&mut self) -> bool {
        self.context_node_kind() == SyntaxKind::ImportType
    }

    pub(crate) fn is_non_jsx_same_line_token_context(&mut self) -> bool {
        self.tokens_are_on_same_line() && self.context_node_kind() != SyntaxKind::JsxText
    }

    pub(crate) fn is_non_jsx_text_context(&mut self) -> bool {
        self.context_node_kind() != SyntaxKind::JsxText
    }

    pub(crate) fn is_non_jsx_element_or_fragment_context(&mut self) -> bool {
        !matches!(
            self.context_node_kind(),
            SyntaxKind::JsxElement | SyntaxKind::JsxFragment
        )
    }

    pub(crate) fn is_jsx_expression_context(&mut self) -> bool {
        matches!(
            self.context_node_kind(),
            SyntaxKind::JsxExpression | SyntaxKind::JsxSpreadAttribute
        )
    }

    pub(crate) fn is_next_token_parent_jsx_attribute(&mut self) -> bool {
        let Some(parent) = &self.next_token_parent else { return false };
        parent.kind == SyntaxKind::JsxAttribute
            || (parent.kind == SyntaxKind::JsxNamespacedName
                && parent.parent.as_ref().is_some_and(|p| p.kind == SyntaxKind::JsxAttribute))
    }

    pub(crate) fn is_jsx_attribute_context(&mut self) -> bool {
        self.context_node_kind() == SyntaxKind::JsxAttribute
    }

    pub(crate) fn is_next_token_parent_not_jsx_namespaced_name(&mut self) -> bool {
        self.next_token_parent.as_ref().map(|p| p.kind) != Some(SyntaxKind::JsxNamespacedName)
    }

    pub(crate) fn is_next_token_parent_jsx_namespaced_name(&mut self) -> bool {
        self.next_token_parent.as_ref().map(|p| p.kind) == Some(SyntaxKind::JsxNamespacedName)
    }

    pub(crate) fn is_jsx_self_closing_element_context(&mut self) -> bool {
        self.context_node_kind() == SyntaxKind::JsxSelfClosingElement
    }

    pub(crate) fn is_not_before_block_in_function_declaration_context(&mut self) -> bool {
        !self.is_function_decl_context() && !self.is_before_block_context()
    }

    pub(crate) fn is_end_of_decorator_context_on_same_line(&mut self) -> bool {
        self.tokens_are_on_same_line()
            && self
                .context_node
                .as_ref()
                .is_some_and(|n| has_decorators(n))
            && self
                .current_token_parent
                .as_ref()
                .is_some_and(|n| node_is_in_decorator_context(n))
            && !self
                .next_token_parent
                .as_ref()
                .is_some_and(|n| node_is_in_decorator_context(n))
    }

    pub(crate) fn is_start_of_variable_declaration_list(&mut self) -> bool {
        let Some(parent) = &self.current_token_parent else { return false };
        parent.kind == SyntaxKind::VariableDeclarationList
            && token_pos_of_node(parent) == self.current_token_span.loc.pos()
    }

    pub(crate) fn is_not_format_on_enter(&mut self) -> bool {
        self.formatting_request_kind != FormatRequestKind::FormatOnEnter
    }

    pub(crate) fn is_module_decl_context(&mut self) -> bool {
        self.context_node_kind() == SyntaxKind::ModuleDeclaration
    }

    pub(crate) fn is_object_type_context(&mut self) -> bool {
        self.context_node_kind() == SyntaxKind::TypeLiteral
    }

    pub(crate) fn is_constructor_signature_context(&mut self) -> bool {
        self.context_node_kind() == SyntaxKind::ConstructSignature
    }

    pub(crate) fn is_type_argument_or_parameter_or_assertion_context(&mut self) -> bool {
        is_type_argument_or_parameter_or_assertion(&self.current_token_span, self.current_token_parent.as_ref())
            || is_type_argument_or_parameter_or_assertion(&self.next_token_span, self.next_token_parent.as_ref())
    }

    pub(crate) fn is_type_assertion_context(&mut self) -> bool {
        self.context_node_kind() == SyntaxKind::TypeAssertionExpression
    }

    pub(crate) fn is_non_type_assertion_context(&mut self) -> bool {
        !self.is_type_assertion_context()
    }

    pub(crate) fn is_void_op_context(&mut self) -> bool {
        self.current_token_span.kind == SyntaxKind::VoidKeyword
            && self
                .current_token_parent
                .as_ref()
                .is_some_and(|p| p.kind == SyntaxKind::VoidExpression)
    }

    pub(crate) fn is_yield_or_yield_star_with_operand(&mut self) -> bool {
        let Some(node) = &self.context_node else { return false };
        if node.kind != SyntaxKind::YieldExpression {
            return false;
        }
        if let crate::ast::NodeData::YieldExpression(d) = &node.data {
            d.expression.is_some()
        } else {
            false
        }
    }

    pub(crate) fn is_non_null_assertion_context(&mut self) -> bool {
        self.context_node_kind() == SyntaxKind::NonNullExpression
    }

    pub(crate) fn is_statement_condition_context(&mut self) -> bool {
        matches!(
            self.context_node_kind(),
            SyntaxKind::IfStatement
                | SyntaxKind::ForStatement
                | SyntaxKind::ForInStatement
                | SyntaxKind::ForOfStatement
                | SyntaxKind::DoStatement
                | SyntaxKind::WhileStatement
        )
    }

    pub(crate) fn is_not_statement_condition_context(&mut self) -> bool {
        !self.is_statement_condition_context()
    }

    pub(crate) fn is_optional_property_context(&self) -> bool {
        let Some(node) = &self.context_node else { return false };
        if node.kind != SyntaxKind::PropertyDeclaration {
            return false;
        }
        if let NodeData::PropertyDeclaration(d) = &node.data {
            return d
                .postfix_token
                .as_ref()
                .is_some_and(|t| t.kind == SyntaxKind::QuestionToken);
        }
        false
    }

    pub(crate) fn is_non_optional_property_context(&mut self) -> bool {
        !self.is_optional_property_context()
    }

    pub(crate) fn is_not_property_access_on_integer_literal(&mut self) -> bool {
        let Some(node) = &self.context_node else { return true };
        if node.kind != SyntaxKind::PropertyAccessExpression {
            return true;
        }
        if let crate::ast::NodeData::PropertyAccessExpression(d) = &node.data {
            if d.expression.kind != SyntaxKind::NumericLiteral {
                return true;
            }
            return d.expression.text().contains('.');
        }
        true
    }

    pub(crate) fn is_semicolon_deletion_context(&mut self) -> bool {
        // 忠实移植 isSemicolonDeletionContext；FindNextToken 简化为取
        // next parent 的第一个 token（格式化场景 parent 链完整）
        let mut next_token_kind = self.next_token_span.kind;
        let mut next_token_start = self.next_token_span.loc.pos();
        if crate::ast::node_data_generated::is_trivia_kind(next_token_kind) {
            let same_parent = match (&self.next_token_parent, &self.current_token_parent) {
                (Some(n), Some(c)) => Arc::ptr_eq(n, c),
                _ => false,
            };
            let next_real = if same_parent {
                find_next_token(self.next_token_parent.as_ref())
            } else {
                self.next_token_parent.clone()
            };
            let Some(next_real) = next_real else {
                return true;
            };
            next_token_kind = next_real.kind;
            next_token_start = token_pos_of_node(&next_real);
        }

        let text = &self.source_file.text;
        let line_of = |pos: usize| text.as_bytes()[..pos.min(text.len())].iter().filter(|&&b| b == b'\n').count();
        if line_of(self.current_token_span.loc.pos()) == line_of(next_token_start) {
            return next_token_kind == SyntaxKind::CloseBraceToken
                || next_token_kind == SyntaxKind::EndOfFile;
        }

        if next_token_kind == SyntaxKind::SemicolonToken
            && self.current_token_span.kind == SyntaxKind::SemicolonToken
        {
            return true;
        }

        if next_token_kind == SyntaxKind::SemicolonClassElement
            || next_token_kind == SyntaxKind::SemicolonToken
        {
            return false;
        }

        if matches!(
            self.context_node_kind(),
            SyntaxKind::InterfaceDeclaration | SyntaxKind::TypeAliasDeclaration
        ) {
            let is_property_sig = self
                .current_token_parent
                .as_ref()
                .is_some_and(|p| p.kind == SyntaxKind::PropertySignature);
            let has_type = self
                .current_token_parent
                .as_ref()
                .and_then(|p| p.name())
                .is_some();
            return !is_property_sig || has_type || next_token_kind != SyntaxKind::OpenParenToken;
        }

        if let Some(parent) = &self.current_token_parent
            && parent.kind == SyntaxKind::PropertyDeclaration
            && let NodeData::PropertyDeclaration(d) = &parent.data
        {
            return d.initializer.is_none();
        }

        !matches!(
            self.current_token_parent.as_ref().map(|p| p.kind),
            Some(SyntaxKind::ForStatement)
                | Some(SyntaxKind::EmptyStatement)
                | Some(SyntaxKind::SemicolonClassElement)
        ) && !matches!(
            next_token_kind,
            SyntaxKind::OpenBracketToken
                | SyntaxKind::OpenParenToken
                | SyntaxKind::PlusToken
                | SyntaxKind::MinusToken
                | SyntaxKind::SlashToken
                | SyntaxKind::RegularExpressionLiteral
                | SyntaxKind::CommaToken
                | SyntaxKind::TemplateExpression
                | SyntaxKind::TemplateHead
                | SyntaxKind::NoSubstitutionTemplateLiteral
                | SyntaxKind::DotToken
        )
    }
}

use crate::ast::NodeData;

fn context_kind_of(context: &FormattingContext) -> SyntaxKind {
    context.context_node.as_ref().map(|n| n.kind).unwrap_or(SyntaxKind::Unknown)
}

/// 为了方法化便利挂到 impl 外的工具（避免命名冲突）
impl FormattingContext {
    pub(crate) fn context_node_kind(&mut self) -> SyntaxKind {
        context_kind_of(self)
    }
}

fn node_is_block_context(node: &Arc<Node>) -> bool {
    if node_is_type_script_decl_with_block(node) {
        return true;
    }
    matches!(
        node.kind,
        SyntaxKind::Block
            | SyntaxKind::CaseBlock
            | SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::ModuleBlock
    )
}

fn node_is_type_script_decl_with_block(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::TypeLiteral
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::NamedExports
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::NamedImports
    )
}

fn node_is_in_decorator_context(node: &Arc<Node>) -> bool {
    let mut cur = Some(node.clone());
    while let Some(n) = cur {
        if !crate::ast::utilities_expressions::is_expression(&n) {
            return n.kind == SyntaxKind::Decorator;
        }
        cur = n.parent.clone();
    }
    false
}

fn has_decorators(node: &Arc<Node>) -> bool {
    if let Some(mods) = node.modifiers() {
        return mods.flags().contains(crate::ast::ModifierFlags::Decorator);
    }
    false
}

pub(crate) fn token_pos_of_node(node: &Arc<Node>) -> usize {
    node.name().map(|n| n.pos()).unwrap_or_else(|| node.pos())
}

fn is_type_argument_or_parameter_or_assertion(
    token: &TextRangeWithKind,
    parent: Option<&Arc<Node>>,
) -> bool {
    if token.kind != SyntaxKind::LessThanToken && token.kind != SyntaxKind::GreaterThanToken {
        return false;
    }
    let Some(parent) = parent else { return false };
    matches!(
        parent.kind,
        SyntaxKind::TypeReference
            | SyntaxKind::TypeAssertionExpression
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::CallSignature
            | SyntaxKind::ConstructSignature
            | SyntaxKind::CallExpression
            | SyntaxKind::NewExpression
            | SyntaxKind::ExpressionWithTypeArguments
    )
}

fn find_next_token(node: Option<&Arc<Node>>) -> Option<Arc<Node>> {
    let node = node?;
    let mut hit = None;
    crate::ast::node_data_generated::for_each_child(node, |c| {
        hit = Some(c.clone());
        true
    });
    hit
}
