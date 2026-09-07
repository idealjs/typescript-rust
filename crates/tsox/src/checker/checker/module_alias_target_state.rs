#![allow(unused_imports)]

use super::*;

pub(crate) fn module_alias_target_state(
    spec: &Arc<Node>,
    export_decl: &Arc<Node>,
    visited: &mut Vec<usize>,
) -> u8 {
    let crate::ast::NodeData::ExportSpecifier(es) = &spec.data else {
        return 2;
    };
    let target_name = es.property_name.as_ref().unwrap_or(&es.name);
    if target_name.kind != SyntaxKind::Identifier {
        return 2;
    }
    let target_text = target_name.text();
    let mut anc = export_decl.parent.as_ref();
    while let Some(p) = anc {
        if matches!(
            p.kind,
            SyntaxKind::ModuleBlock | SyntaxKind::Block | SyntaxKind::SourceFile
        ) {
            let stmts: &[Arc<Node>] = match &p.data {
                crate::ast::NodeData::ModuleBlock(b) => &b.statements.nodes,
                crate::ast::NodeData::SourceFile(sf) => &sf.statements.nodes,
                crate::ast::NodeData::Block(b) => &b.statements.nodes,
                _ => &[],
            };
            let mut found: Option<u8> = None;
            for s in stmts {
                if statement_declares_name(s, target_text) {
                    let st = module_instance_state(s, visited);
                    found = Some(found.map_or(st, |f| f.max(st)));
                    if found == Some(2) {
                        return 2;
                    }
                    if s.kind == SyntaxKind::ImportEqualsDeclaration {
                        return 2;
                    }
                }
            }
            if let Some(f) = found {
                return f;
            }
        }
        anc = p.parent.as_ref();
    }
    2
}

pub(crate) fn statement_declares_name(stmt: &Arc<Node>, id_text: &str) -> bool {
    let name: Option<&Arc<Node>> = match &stmt.data {
        crate::ast::NodeData::FunctionDeclaration(f) => f.name.as_ref(),
        crate::ast::NodeData::ClassDeclaration(c) => c.name.as_ref(),
        crate::ast::NodeData::EnumDeclaration(e) => Some(&e.name),
        crate::ast::NodeData::ModuleDeclaration(m) => Some(&m.name),
        crate::ast::NodeData::InterfaceDeclaration(i) => Some(&i.name),
        crate::ast::NodeData::TypeAliasDeclaration(t) => Some(&t.name),
        crate::ast::NodeData::ImportEqualsDeclaration(i) => Some(&i.name),
        crate::ast::NodeData::VariableStatement(vs) => {
            let crate::ast::NodeData::VariableDeclarationList(dl) = &vs.declaration_list.data
            else {
                return false;
            };
            return dl
                .declarations
                .nodes
                .iter()
                .any(|d| binding_names_cover(d, id_text));
        }
        _ => None,
    };
    name.is_some_and(|n| n.kind == SyntaxKind::Identifier && n.text() == id_text)
}

pub(crate) fn binding_names_cover(decl: &Arc<Node>, id_text: &str) -> bool {
    let crate::ast::NodeData::VariableDeclaration(d) = &decl.data else {
        return false;
    };
    match &d.name.kind {
        SyntaxKind::Identifier => d.name.text() == id_text,
        SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern => {
            let mut hit = false;
            crate::ast::node_data_generated::for_each_child(&d.name, |el| {
                if binding_names_cover(el, id_text) {
                    hit = true;
                    true
                } else {
                    false
                }
            });
            hit
        }
        _ => false,
    }
}
