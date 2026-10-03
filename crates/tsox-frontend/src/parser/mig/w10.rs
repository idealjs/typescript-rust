#![allow(unused_imports)]

use std::sync::Arc;

use crate::ast::*;
use crate::parser::binary_precedence::is_keyword;
use crate::parser::parsing_context::Parser;
use crate::scanner::Scanner;
use tsox_core::core::text::TextRange;
use tsox_core::diagnostics::{self, Message};

// 孪生/架构裁决（Go 名 → Rust 名/位置）：
// parseToplevelStatement → reparse_await.rs reparse_top_level_await（架构内联：await 上下文以文件级后处理扫描，Parser 无 statementHasAwaitIdentifier/possibleAwaitSpans 字段）
// parseTupleElementNameOrTupleElementType → types_parser_3.rs parse_tuple_element_type（内联：named-tuple 分支并入其中）
// parseTypeOperator → types_parser.rs parse_type_operator_or_higher（内联：operator 节点构造并入 or_higher）
// putParser → wp1.rs 已裁决（无对象池，Parser 字段直构）
// setContextFlags → 架构差异：context flags 以 yield_context/await_context/disallow_in_context 布尔字段表达
// processPragmasIntoFields → wp1_2.rs process_pragmas_into_fields（活体入口 impl_chunk_parser.rs parse_source_file_text_with_diagnostics；CheckJsDirective 无存储未落字段）

impl Parser {
    pub(crate) fn parse_type_annotation(&mut self) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("parse_type_annotation"); 
        if self.parse_optional(SyntaxKind::ColonToken) {
            return Some(self.parse_type());
        }
        None
    }

    pub(crate) fn parse_type_of_expression(&mut self) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_type_of_expression"); 
        self.parse_unary_expression()
    }

    pub(crate) fn parse_type_parameter_of_infer_type(&mut self) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_type_parameter_of_infer_type"); 
        let pos = self.node_pos();
        let name = self.parse_identifier();
        let constraint = self.try_parse_constraint_of_infer_type();
        let end = constraint.as_ref().map_or(name.end(), |n| n.end());
        Arc::new(Node::with_loc(
            SyntaxKind::TypeParameter,
            NodeData::TypeParameterDeclaration(TypeParameterDeclarationData {
                modifiers: None,
                name,
                constraint,
                expression: None,
                default_type: None,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_unary_expression_or_higher(&mut self) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_unary_expression_or_higher"); 
        self.parse_unary_expression()
    }

    pub(crate) fn parse_update_expression(&mut self) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_update_expression"); 
        self.parse_unary_expression()
    }

    pub(crate) fn parse_simple_unary_expression(&mut self) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_simple_unary_expression"); 
        self.parse_unary_expression()
    }

    pub(crate) fn parse_left_hand_side_expression_or_higher(&mut self) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_left_hand_side_expression_or_higher"); 
        self.parse_left_hand_side_expression()
    }

    pub(crate) fn re_scan_greater_than_token(&mut self) -> SyntaxKind { ::tsox_core::fntrace::enter("re_scan_greater_than_token"); 
        self.token = self.scanner.re_scan_greater_than();
        self.token
    }

    pub(crate) fn scan_class_member_start(&mut self) -> bool { ::tsox_core::fntrace::enter("scan_class_member_start"); 
        let mut id_token = SyntaxKind::Unknown;
        if self.token == SyntaxKind::AtToken {
            return true;
        }
        while is_modifier_kind(self.token) {
            id_token = self.token;
            if crate::ast::mig::m3f_4::is_class_member_modifier(id_token) {
                return true;
            }
            self.next_token();
        }
        if self.token == SyntaxKind::AsteriskToken {
            return true;
        }
        if self.is_literal_property_name() {
            id_token = self.token;
            self.next_token();
        }
        if self.token == SyntaxKind::OpenBracketToken {
            return true;
        }
        if id_token != SyntaxKind::Unknown {
            if !is_keyword(id_token)
                || id_token == SyntaxKind::SetKeyword
                || id_token == SyntaxKind::GetKeyword
            {
                return true;
            }
            match self.token {
                SyntaxKind::OpenParenToken
                | SyntaxKind::LessThanToken
                | SyntaxKind::ExclamationToken
                | SyntaxKind::ColonToken
                | SyntaxKind::EqualsToken
                | SyntaxKind::QuestionToken => return true,
                _ => return self.can_parse_semicolon(),
            }
        }
        false
    }

    pub(crate) fn scan_type_member_start(&mut self) -> bool { ::tsox_core::fntrace::enter("scan_type_member_start"); 
        if self.token == SyntaxKind::OpenParenToken
            || self.token == SyntaxKind::LessThanToken
            || self.token == SyntaxKind::GetKeyword
            || self.token == SyntaxKind::SetKeyword
        {
            return true;
        }
        let mut id_token = false;
        while is_modifier_kind(self.token) {
            id_token = true;
            self.next_token();
        }
        if self.token == SyntaxKind::OpenBracketToken {
            return true;
        }
        if self.is_literal_property_name() {
            id_token = true;
            self.next_token();
        }
        if id_token {
            return self.token == SyntaxKind::OpenParenToken
                || self.token == SyntaxKind::LessThanToken
                || self.token == SyntaxKind::QuestionToken
                || self.token == SyntaxKind::ColonToken
                || self.token == SyntaxKind::CommaToken
                || self.can_parse_semicolon();
        }
        false
    }
}
