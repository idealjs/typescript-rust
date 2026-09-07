#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn is_scope_exempt(
        &mut self,
        node: &Arc<Node>,
        declaration_for_scope: Option<&Arc<Node>>,
    ) -> bool {
        if let Some(declaration_for_scope) = declaration_for_scope {
            let is_fn_like = |n: &Arc<Node>| {
                matches!(
                    n.kind,
                    SyntaxKind::FunctionDeclaration
                        | SyntaxKind::FunctionExpression
                        | SyntaxKind::ArrowFunction
                        | SyntaxKind::MethodDeclaration
                        | SyntaxKind::Constructor
                        | SyntaxKind::GetAccessor
                        | SyntaxKind::SetAccessor
                )
            };
            let immediately_invoked = |n: &Arc<Node>| -> bool {
                let Some(p) = n.parent.as_ref() else {
                    return false;
                };
                match &p.data {
                    crate::ast::NodeData::CallExpression(_) => true,
                    crate::ast::NodeData::ParenthesizedExpression(_) => {
                        let mut cur = p.parent.as_ref();
                        while let Some(a) = cur {
                            if matches!(&a.data, crate::ast::NodeData::CallExpression(_)) {
                                return true;
                            }
                            if matches!(&a.data, crate::ast::NodeData::ParenthesizedExpression(_)) {
                                cur = a.parent.as_ref();
                                continue;
                            }
                            break;
                        }
                        false
                    }
                    _ => false,
                }
            };

            let mut dc = declaration_for_scope.parent.as_ref();
            let mut decl_container: Option<Arc<Node>> = None;
            while let Some(a) = dc {
                if is_fn_like(a) {
                    decl_container = Some(Arc::clone(a));
                    break;
                }
                dc = a.parent.as_ref();
            }
            let mut cur = node.parent.as_ref();
            let mut exempt = false;
            while let Some(a) = cur {
                if let Some(dcont) = &decl_container {
                    if Arc::ptr_eq(a, dcont) {
                        break;
                    }
                }
                if is_fn_like(a) {
                    if immediately_invoked(a) {
                        cur = a.parent.as_ref();
                        continue;
                    }
                    exempt = true;
                    break;
                }

                if a.kind == SyntaxKind::PropertyDeclaration {
                    let in_initializer = matches!(&a.data, crate::ast::NodeData::PropertyDeclaration(pd) if pd.initializer.as_ref().is_some_and(|init| init.loc.contains(node.loc.pos())));
                    let is_static_prop = a.has_syntactic_modifier(ModifierFlags::Static);
                    let is_decl_instance_prop = declaration_for_scope.kind
                        == SyntaxKind::PropertyDeclaration
                        && !declaration_for_scope.has_syntactic_modifier(ModifierFlags::Static);
                    if in_initializer && !is_static_prop && !is_decl_instance_prop {
                        exempt = true;
                        break;
                    }
                }
                cur = a.parent.as_ref();
            }
            if exempt {
                return true;
            }
        }
        false
    }
}
