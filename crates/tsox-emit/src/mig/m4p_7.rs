#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node::{Node, SourceFile};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

use tsox_frontend::format::mig::m4o_2::compare_emit_helpers;
use tsox_frontend::format::mig::m4o_2::WriteKind;
use tsox_frontend::format::mig::m4o::ListFormat;

use super::m4p::Printer;
use super::m4p::r39k22_defs::{LF_HERITAGE_CLAUSE_TYPES, R39k22NodeExt};

impl Printer {
    pub fn emit_heritage_clause(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_heritage_clause"); 
        let state = self.enter_node(node);
        self.write_space();
        self.emit_token(node.heritage_token22(), node.pos(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_list(
            Self::emit_heritage_clause_element,
            node,
            node.heritage_types22(),
            LF_HERITAGE_CLAUSE_TYPES,
        );
        self.exit_node(node, state);
    }

    pub fn emit_heritage_clause_element(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_heritage_clause_element"); 
        match node.kind {
            SyntaxKind::ExpressionWithTypeArguments => self.emit_expression_with_type_arguments(node),
            SyntaxKind::TypeReference => self.emit_type_reference(node),
            _ => panic!("unhandled HeritageClauseElement: {:?}", node.kind),
        }
    }

    pub fn emit_heritage_clause_node(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_heritage_clause_node"); 
        self.emit_heritage_clause(node);
    }

    pub fn emit_enum_member(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_enum_member"); 
        let state = self.enter_node(node);
        let name = node.name().unwrap();
        self.emit_property_name(name);
        self.emit_initializer(node.initializer(), name.end(), node);
        self.exit_node(node, state);
    }

    pub fn emit_enum_member_node(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_enum_member_node"); 
        self.emit_enum_member(node);
    }

    pub fn emit_jsdoc_node(&mut self, _node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsdoc_node"); 
        panic!("not implemented");
    }

    pub fn emit_helpers(&mut self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("emit_helpers"); 
        let mut helpers_emitted = false;
        let source_file: Option<&Arc<SourceFile>> = self.current_source_file.as_ref();
        let should_skip = self.options.no_emit_helpers
            || source_file
                .is_some_and(|f| self.emit_context.has_recorded_external_helpers(&f.node));
        let mut helpers = self.emit_context.get_emit_helpers(node).to_vec();
        if !helpers.is_empty() {
            helpers.sort_by(|a, b| compare_emit_helpers(a, b).cmp(&0));
            for helper in &helpers {
                if !helper.scoped && should_skip {
                    continue;
                }
                if let Some(text_callback) = &helper.text_callback {
                    let emit_context = self.emit_context.clone();
                    let text = text_callback(&|name: &str| {
                        emit_context
                            .factory()
                            .new_unique_name_ex(
                                name,
                                crate::printer::generated_identifier_flags::AutoGenerateOptions {
                                    flags: crate::printer::generated_identifier_flags::GeneratedIdentifierFlags::FILE_LEVEL
                                        | crate::printer::generated_identifier_flags::GeneratedIdentifierFlags::OPTIMISTIC,
                                    prefix: String::new(),
                                    suffix: String::new(),
                                },
                            )
                            .text()
                            .to_string()
                    });
                    self.write_lines(&text);
                } else {
                    self.write_lines(&helper.text);
                }
                helpers_emitted = true;
            }
        }
        helpers_emitted
    }
}
