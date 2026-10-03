#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use tsox_core::core::text::TextRange;
#[path = "r39k12_defs.rs"]
pub mod r39k12_defs;
pub use r39k12_defs::R39K12EmitContextMapsExt;

use super::m4n::r37k19_defs::R37K19ArcNodeExt;
use crate::mig::m4f_3::r38k10_defs::R38K10NodeVisitorExt;
use crate::mig::m4j::r36k3_defs::R36K3NodeAccessExt;
use crate::mig::m4n_5::r36k29_defs::R36K29NodeExt;
use super::m4q::r33k12_defs::is_parse_tree_node;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{
    is_binding_pattern, is_block, is_not_emitted_statement,
};
use tsox_frontend::ast::node::NodeList;
use tsox_frontend::ast::visitor::NodeVisitor;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

use crate::printer::mig::m4m_2::{
    EmitNode, HAS_COMMENT_RANGE, HAS_SOURCE_MAP_RANGE, SnippetElement, SynthesizedComment,
};
use crate::printer::EmitContext;
use super::m4q::r33k12_defs::EmitFlags;
use crate::printer::mig::m4m_3::{
    VarScope, ENVIRONMENT_FLAGS_IN_PARAMETERS, ENVIRONMENT_FLAGS_VARIABLES_HOISTED_IN_PARAMETERS,
};

impl EmitContext {
    pub fn set_assigned_name(&mut self, node: &Arc<Node>, name: &Arc<Node>) { ::tsox_core::fntrace::enter("set_assigned_name"); 
        self.r39k12_assigned_name_set(node, name);
    }

    pub fn set_class_this(&mut self, node: &Arc<Node>, class_this: &Arc<Node>) { ::tsox_core::fntrace::enter("set_class_this"); 
        self.r39k12_class_this_set(node, class_this);
    }

    pub fn set_comment_range(&mut self, node: &Arc<Node>, loc: TextRange) { ::tsox_core::fntrace::enter("set_comment_range"); 
        let mut emit_node = self.emit_nodes_get_mut(node);
        emit_node.comment_range = loc;
        emit_node.flags |= HAS_COMMENT_RANGE;
    }

    pub fn set_emit_flags(&mut self, node: &Arc<Node>, flags: EmitFlags) { ::tsox_core::fntrace::enter("set_emit_flags"); 
        self.emit_nodes_get_mut(node).emit_flags = flags;
    }

    pub fn set_external_helpers_module_name(&mut self, node: &Arc<Node>, name: &Arc<Node>) { ::tsox_core::fntrace::enter("set_external_helpers_module_name"); 
        let Some(parse_node) = self.parse_node(node) else {
            panic!("Node must be a parse tree node or have an Original pointer to a parse tree node.")
        };
        self.emit_nodes_get_mut(&parse_node).external_helpers_module_name = Some(Arc::clone(name));
    }

    pub fn set_original(&self, node: &Arc<Node>, original: &Arc<Node>) { ::tsox_core::fntrace::enter("set_original"); 
        self.set_original_ex(node, original, false);
    }

    pub fn set_original_ex(
        &self,
        node: &Arc<Node>,
        original: &Arc<Node>,
        allow_overwrite: bool,
    ) { ::tsox_core::fntrace::enter("set_original_ex"); 
        match self.r39k12_original_get(node) {
            None => {
                self.r39k12_original_set(node, original);
                let copied = self
                    .emit_nodes_try_get(original)
                    .map(|emit_node| emit_node.clone());
                if let Some(copied) = copied {
                    let mut target = self.emit_nodes_get_mut(node);
                    target.flags = copied.flags;
                    target.emit_flags = copied.emit_flags;
                    target.comment_range = copied.comment_range;
                    target.source_map_range = copied.source_map_range;
                    target.token_source_map_ranges = copied.token_source_map_ranges.clone();
                    target.helpers = copied.helpers.clone();
                    target.external_helpers_module_name = copied.external_helpers_module_name.clone();
                    if let Some(snippet_element) = &copied.snippet_element {
                        target.snippet_element = Some(snippet_element.clone());
                    }
                }
            }
            Some(existing) => {
                if !allow_overwrite && !Arc::ptr_eq(&existing, original) {
                    panic!("Original node already set.");
                }
                if allow_overwrite {
                    self.r39k12_original_set(node, original);
                }
            }
        }
    }

    pub fn set_snippet_element(&mut self, node: &Arc<Node>, snippet_element: SnippetElement) { ::tsox_core::fntrace::enter("set_snippet_element"); 
        self.emit_nodes_get_mut(node).snippet_element = Some(snippet_element);
    }

    pub fn set_source_map_range(&mut self, node: &Arc<Node>, loc: TextRange) { ::tsox_core::fntrace::enter("set_source_map_range"); 
        let mut emit_node = self.emit_nodes_get_mut(node);
        emit_node.source_map_range = loc;
        emit_node.flags |= HAS_SOURCE_MAP_RANGE;
    }

    pub fn set_synthetic_leading_comments(
        &mut self,
        node: &Arc<Node>,
        comments: Vec<SynthesizedComment>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("set_synthetic_leading_comments"); 
        self.emit_nodes_get_mut(node).leading_comments = comments;
        Arc::clone(node)
    }

    pub fn set_synthetic_trailing_comments(
        &mut self,
        node: &Arc<Node>,
        comments: Vec<SynthesizedComment>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("set_synthetic_trailing_comments"); 
        self.emit_nodes_get_mut(node).trailing_comments = comments;
        Arc::clone(node)
    }

    pub fn set_token_source_map_range(
        &mut self,
        node: &Arc<Node>,
        kind: SyntaxKind,
        loc: TextRange,
    ) { ::tsox_core::fntrace::enter("set_token_source_map_range"); 
        self.emit_nodes_get_mut(node)
            .token_source_map_ranges
            .insert(kind, loc);
    }

    pub fn set_type_node(&mut self, node: &Arc<Node>, type_node: &Arc<Node>) { ::tsox_core::fntrace::enter("set_type_node"); 
        self.emit_nodes_get_mut(node).type_node = Some(Arc::clone(type_node));
    }

    pub fn snippet_element(&self, node: &Arc<Node>) -> Option<SnippetElement> { ::tsox_core::fntrace::enter("snippet_element"); 
        self.emit_nodes_try_get(node).and_then(|e| e.snippet_element.clone())
    }

    pub fn source_map_range(&self, node: &Arc<Node>) -> TextRange { ::tsox_core::fntrace::enter("source_map_range"); 
        if let Some(emit_node) = self.emit_nodes_try_get(node) {
            if emit_node.flags & HAS_SOURCE_MAP_RANGE != 0 {
                return emit_node.source_map_range;
            }
        }
        node.loc
    }

    pub fn start_lexical_environment(&mut self) { ::tsox_core::fntrace::enter("start_lexical_environment"); 
        self.let_scope_stack.push(VarScope::default());
    }

    pub fn start_variable_environment(&mut self) { ::tsox_core::fntrace::enter("start_variable_environment"); 
        self.var_scope_stack.push(VarScope::default());
        self.start_lexical_environment();
    }

    pub fn text_source(&self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("text_source"); 
        self.r39k12_text_source_get(node)
    }

    pub fn token_source_map_range(&self, node: &Arc<Node>, kind: SyntaxKind) -> Option<TextRange> { ::tsox_core::fntrace::enter("token_source_map_range"); 
        self.emit_nodes_try_get(node)
            .and_then(|emit_node| emit_node.token_source_map_ranges.get(&kind).copied())
    }

    pub fn unset_original(&self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("unset_original"); 
        self.r39k12_original_remove(node);
    }

    pub fn visit_embedded_statement(
        &mut self,
        node: &Arc<Node>,
        visitor: &mut NodeVisitor,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_embedded_statement"); 
        if let Some(embedded_statement) = visitor.visit_embedded_statement(node) {
            if !is_not_emitted_statement(&embedded_statement) {
                return Some(embedded_statement);
            }
        }
        let mut empty_statement = self.factory().new_empty_statement();
        empty_statement.set_loc(node.loc);
        self.set_original(&empty_statement, node);
        self.assign_comment_range(&empty_statement, node);
        Some(empty_statement)
    }

    pub fn visit_function_body(
        &mut self,
        node: Option<Arc<Node>>,
        visitor: &mut NodeVisitor,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_function_body"); 
        let updated = visitor.visit_node_opt(node.as_ref());
        let declarations = self.end_variable_environment();
        if declarations.is_empty() {
            return updated;
        }

        let Some(updated) = updated else {
            return Some(
                self.factory()
                    .new_block(&self.factory().new_node_list(declarations), true),
            );
        };

        if !is_block(&updated) {
            self.add_emit_flags(&updated, EmitFlags::NO_COMMENTS);
            let block = self.convert_to_function_block(&updated, false);
            let block_statements = tsox_frontend::ast::mig::m3c::statement_list(&block)
                .expect("block requires a statement list")
                .clone();
            let merged = self.merge_environment_list(&block_statements, declarations);
            return Some(self.factory().update_block(
                &block,
                &merged,
                block.as_block().multi_line,
            ));
        }

        let updated_statements = tsox_frontend::ast::mig::m3c::statement_list(&updated)
            .expect("block requires a statement list")
            .clone();
        let merged = self.merge_environment_list(&updated_statements, declarations);
        Some(self.factory().update_block(
            &updated,
            &merged,
            updated.as_block().multi_line,
        ))
    }

    pub fn visit_iteration_body(
        &mut self,
        body: Option<Arc<Node>>,
        visitor: &mut NodeVisitor,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_iteration_body"); 
        let body = body?;

        self.start_lexical_environment();
        let updated = self.visit_embedded_statement(&body, visitor);
        let Some(updated) = updated else {
            panic!("Expected visitor to return a statement.");
        };

        let mut statements = self.end_lexical_environment();
        if !statements.is_empty() {
            if is_block(&updated) {
                statements.extend(updated.statements());
                let updated_list_loc = tsox_frontend::ast::mig::m3c::statement_list(&updated)
                    .expect("block requires a statement list")
                    .loc;
                let mut statements_list = self.factory().new_node_list(statements);
                if let Some(list) = Arc::get_mut(&mut statements_list) {
                    list.loc = updated_list_loc;
                }
                return Some(self.factory().update_block(
                    &updated,
                    &statements_list,
                    updated.as_block().multi_line,
                ));
            }
            statements.push(updated);
            return Some(
                self.factory()
                    .new_block(&self.factory().new_node_list(statements), true),
            );
        }

        Some(updated)
    }

    pub fn visit_parameters(
        &mut self,
        nodes: &NodeList,
        visitor: &mut NodeVisitor,
    ) -> NodeList { ::tsox_core::fntrace::enter("visit_parameters"); 
        self.start_variable_environment();
        let scope = self.var_scope_stack.last_mut().expect("variable scope");
        let old_flags = scope.flags;
        scope.flags |= ENVIRONMENT_FLAGS_IN_PARAMETERS;
        let mut nodes = visitor.visit_nodes(nodes);

        if self.var_scope_stack.last().expect("variable scope").flags
            & ENVIRONMENT_FLAGS_VARIABLES_HOISTED_IN_PARAMETERS
            != 0
        {
            nodes = self.add_default_value_assignments_if_needed(Some(&nodes));
        }
        self.var_scope_stack.last_mut().expect("variable scope").flags = old_flags;
        nodes
    }

    pub fn visit_variable_environment(
        &mut self,
        nodes: &NodeList,
        visitor: &mut NodeVisitor,
    ) -> NodeList { ::tsox_core::fntrace::enter("visit_variable_environment"); 
        self.start_variable_environment();
        let visited = visitor.visit_nodes(nodes);
        self.end_and_merge_variable_environment_list(Some(&visited))
            .unwrap_or(visited)
    }
}
