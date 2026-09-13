use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::node_data_generated::{is_identifier, NodeData};
use crate::ast::node_flags::ModifierFlags;
use crate::ast::node::Node;
use crate::ast::syntax_kind_generated::SyntaxKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModuleInstanceState {
    Unknown,
    NonInstantiated,
    Instantiated,
    ConstEnumOnly,
}

pub fn get_module_instance_state(node: &Arc<Node>) -> ModuleInstanceState {
    if let NodeData::ModuleDeclaration(md) = &node.data {
        if let Some(body) = &md.body {
            let mut visited = HashMap::new();
            return get_module_instance_state_cached(body, &mut visited);
        }
    }
    ModuleInstanceState::Instantiated
}

pub fn is_instantiated_module(node: &Arc<Node>, preserve_const_enums: bool) -> bool {
    let state = get_module_instance_state(node);
    state == ModuleInstanceState::Instantiated
        || (preserve_const_enums && state == ModuleInstanceState::ConstEnumOnly)
}

fn get_module_instance_state_cached(
    node: &Arc<Node>,
    visited: &mut HashMap<u64, ModuleInstanceState>,
) -> ModuleInstanceState {
    let node_id = node.id();
    if let Some(cached) = visited.get(&node_id) {
        return if *cached != ModuleInstanceState::Unknown {
            *cached
        } else {
            ModuleInstanceState::NonInstantiated
        };
    }
    visited.insert(node_id, ModuleInstanceState::Unknown);
    let result = get_module_instance_state_worker(node, visited);
    visited.insert(node_id, result);
    result
}

fn get_module_instance_state_worker(
    node: &Arc<Node>,
    visited: &mut HashMap<u64, ModuleInstanceState>,
) -> ModuleInstanceState {
    match node.kind {
        SyntaxKind::InterfaceDeclaration | SyntaxKind::TypeAliasDeclaration
        | SyntaxKind::JSTypeAliasDeclaration => return ModuleInstanceState::NonInstantiated,
        SyntaxKind::EnumDeclaration => {
            if node.has_syntactic_modifier(ModifierFlags::Const) {
                return ModuleInstanceState::ConstEnumOnly;
            }
        }
        SyntaxKind::ImportDeclaration | SyntaxKind::ImportEqualsDeclaration => {
            if !node.has_syntactic_modifier(ModifierFlags::Export) {
                return ModuleInstanceState::NonInstantiated;
            }
        }
        SyntaxKind::ExportDeclaration => {
            if let NodeData::ExportDeclaration(decl) = &node.data {
                if decl.module_specifier.is_none() {
                    if let Some(export_clause) = &decl.export_clause {
                        if export_clause.kind == SyntaxKind::NamedExports {
                            let mut state = ModuleInstanceState::NonInstantiated;
                            if let NodeData::NamedExports(named) = &export_clause.data {
                                for specifier in &named.elements.nodes {
                                    let specifier_state =
                                        get_module_instance_state_for_alias_target(
                                            specifier,
                                            visited,
                                        );
                                    if specifier_state > state {
                                        state = specifier_state;
                                    }
                                    if state == ModuleInstanceState::Instantiated {
                                        return state;
                                    }
                                }
                            }
                            return state;
                        }
                    }
                }
            }
        }
        SyntaxKind::ModuleBlock => {
            let mut state = ModuleInstanceState::NonInstantiated;
            if let NodeData::ModuleBlock(block) = &node.data {
                for child in &block.statements.nodes {
                    let child_state = get_module_instance_state_cached(child, visited);
                    match child_state {
                        ModuleInstanceState::NonInstantiated => continue,
                        ModuleInstanceState::ConstEnumOnly => {
                            state = ModuleInstanceState::ConstEnumOnly;
                        }
                        ModuleInstanceState::Instantiated => {
                            return ModuleInstanceState::Instantiated;
                        }
                        ModuleInstanceState::Unknown => {}
                    }
                }
            }
            return state;
        }
        SyntaxKind::ModuleDeclaration => return get_module_instance_state(node),
        _ => {}
    }
    ModuleInstanceState::Instantiated
}

fn get_module_instance_state_for_alias_target(
    node: &Arc<Node>,
    visited: &mut HashMap<u64, ModuleInstanceState>,
) -> ModuleInstanceState {
    let name = if let NodeData::ExportSpecifier(spec) = &node.data {
        spec.property_name.as_ref().unwrap_or(&spec.name)
    } else {
        return ModuleInstanceState::Instantiated;
    };
    if !is_identifier(name) {
        return ModuleInstanceState::Instantiated;
    }
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind == SyntaxKind::Block
            || parent.kind == SyntaxKind::ModuleBlock
            || parent.kind == SyntaxKind::SourceFile
        {
            let statements: &[Arc<Node>] = match &parent.data {
                NodeData::Block(b) => &b.statements.nodes,
                NodeData::ModuleBlock(mb) => &mb.statements.nodes,
                NodeData::SourceFile(sf) => &sf.statements.nodes,
                _ => &[],
            };
            let mut found = ModuleInstanceState::Unknown;
            for statement in statements {
                if node_has_name(statement, name) {
                    let state = get_module_instance_state_cached(statement, visited);
                    if found == ModuleInstanceState::Unknown || state > found {
                        found = state;
                    }
                    if found == ModuleInstanceState::Instantiated {
                        return found;
                    }
                    if statement.kind == SyntaxKind::ImportEqualsDeclaration {
                        found = ModuleInstanceState::Instantiated;
                    }
                }
            }
            if found != ModuleInstanceState::Unknown {
                return found;
            }
        }
        current = parent.parent();
    }
    ModuleInstanceState::Instantiated
}

fn node_has_name(statement: &Arc<Node>, id: &Arc<Node>) -> bool {
    if let Some(name) = statement.name() {
        return is_identifier(name) && name.text() == id.text();
    }
    if statement.kind == SyntaxKind::VariableStatement {
        if let NodeData::VariableStatement(vs) = &statement.data {
            if let NodeData::VariableDeclarationList(list) = &vs.declaration_list.data {
                for d in &list.declarations.nodes {
                    if node_has_name(d, id) {
                        return true;
                    }
                }
            }
        }
    }
    false
}
