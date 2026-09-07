#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn is_declared_as_plain_var(&self, symbol: &Arc<Symbol>) -> bool {
        let decl = symbol
            .value_declaration
            .as_ref()
            .or_else(|| symbol.declarations.first());
        if let Some(mut current) = decl {
            loop {
                match current.kind {
                    SyntaxKind::VariableDeclaration => {
                        let is_var = current.parent.as_ref().is_some_and(|parent| {
                            parent.kind == SyntaxKind::VariableDeclarationList
                                && !parent.flags.intersects(
                                    crate::ast::NodeFlags::Let | crate::ast::NodeFlags::Const,
                                )
                        });
                        if is_var {
                            return true;
                        }
                        break;
                    }
                    SyntaxKind::BindingElement
                    | SyntaxKind::ObjectBindingPattern
                    | SyntaxKind::ArrayBindingPattern => match current.parent.as_ref() {
                        Some(parent) => current = parent,
                        None => break,
                    },
                    _ => break,
                }
            }
        }
        false
    }

    pub(crate) fn has_const_enum_declarations(&mut self, symbol: &Arc<Symbol>) -> bool {
        let mut enum_decl_count = 0;
        let is_const_enum = symbol
            .declarations
            .iter()
            .filter(|d| {
                if d.kind == SyntaxKind::EnumDeclaration {
                    enum_decl_count += 1;
                    true
                } else {
                    false
                }
            })
            .all(|d| {
                let Some(f) = self
                    .get_source_file_of_node(d)
                    .or_else(|| self.current_file.clone())
                else {
                    return false;
                };
                let text = &f.text;
                let start = d.loc.pos();

                let lo = start.saturating_sub(8);
                let window = &text[lo.min(text.len())..(start + 6).min(text.len())];
                window.contains("const")
            });
        is_const_enum && enum_decl_count > 0 && !self.compiler_options.isolated_modules.is_true()
    }

    pub(crate) fn is_used_in_type_position(&self, node: &Arc<Node>) -> bool {
        let in_tp_default = {
            let mut cur = node.parent.as_ref();
            let mut hit = false;
            while let Some(a) = cur {
                if a.kind == SyntaxKind::TypeParameter {
                    hit = true;
                    break;
                }
                if matches!(
                    a.kind,
                    SyntaxKind::FunctionDeclaration
                        | SyntaxKind::ClassDeclaration
                        | SyntaxKind::MethodDeclaration
                        | SyntaxKind::Block
                        | SyntaxKind::SourceFile
                ) {
                    break;
                }
                cur = a.parent.as_ref();
            }
            hit
        };
        let in_type_position = {
            let mut cur = node.parent.as_ref();
            let mut hit = false;
            while let Some(a) = cur {
                if matches!(
                    a.kind,
                    SyntaxKind::TypeReference
                        | SyntaxKind::TypeParameter
                        | SyntaxKind::ArrayType
                        | SyntaxKind::UnionType
                        | SyntaxKind::IntersectionType
                        | SyntaxKind::ParenthesizedType
                        | SyntaxKind::TupleType
                        | SyntaxKind::TypeLiteral
                        | SyntaxKind::FunctionType
                        | SyntaxKind::ConstructorType
                        | SyntaxKind::QualifiedName
                        | SyntaxKind::HeritageClause
                ) {
                    hit = true;
                    break;
                }
                if matches!(
                    a.kind,
                    SyntaxKind::FunctionDeclaration
                        | SyntaxKind::ClassDeclaration
                        | SyntaxKind::MethodDeclaration
                        | SyntaxKind::Block
                        | SyntaxKind::SourceFile
                ) {
                    break;
                }
                cur = a.parent.as_ref();
            }
            hit
        };
        in_tp_default || in_type_position
    }
}
