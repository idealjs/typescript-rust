#![allow(unused_imports)]

use crate::checker::checker_contextual::*;

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
                let Some(p) = n.parent() else {
                    return false;
                };
                match &p.data {
                    tsox_frontend::ast::NodeData::CallExpression(_) => true,
                    tsox_frontend::ast::NodeData::ParenthesizedExpression(_) => {
                        let mut cur = p.parent();
                        while let Some(a) = cur {
                            if matches!(&a.data, tsox_frontend::ast::NodeData::CallExpression(_)) {
                                return true;
                            }
                            if matches!(
                                &a.data,
                                tsox_frontend::ast::NodeData::ParenthesizedExpression(_)
                            ) {
                                cur = a.parent();
                                continue;
                            }
                            break;
                        }
                        false
                    }
                    _ => false,
                }
            };

            let mut dc = declaration_for_scope.parent();
            let mut decl_container: Option<Arc<Node>> = None;
            while let Some(a) = dc {
                if is_fn_like(&a) {
                    decl_container = Some(Arc::clone(&a));
                    break;
                }
                dc = a.parent();
            }
            let mut cur = node.parent();
            let mut exempt = false;
            'walk: while let Some(a) = cur {
                if let Some(dcont) = &decl_container {
                    if Arc::ptr_eq(&a, dcont) {
                        break;
                    }
                }
                // Go isUsedInFunctionOrInstanceProperty 装饰器分支：装饰器
                // 表达式整体位于方法/参数装饰器内时，改从被装饰成员的容器续走，
                // 不把宿主方法当函数边界豁免
                if let Some(parent) = a.parent()
                    && parent.kind == SyntaxKind::Decorator
                    && let tsox_frontend::ast::NodeData::Decorator(d) = &parent.data
                    && Arc::ptr_eq(&d.expression, &a)
                {
                    match parent.parent().map(|decorated| (Arc::clone(&decorated), decorated.kind))
                    {
                        Some((decorated, SyntaxKind::MethodDeclaration)) => {
                            cur = decorated.parent();
                            continue 'walk;
                        }
                        Some((decorated, SyntaxKind::Parameter)) => {
                            let mut boundary = decorated.parent();
                            let mut landed: Option<Arc<Node>> = None;
                            while let Some(b) = boundary {
                                landed = Some(Arc::clone(&b));
                                if matches!(
                                    b.kind,
                                    SyntaxKind::MethodDeclaration
                                        | SyntaxKind::Constructor
                                        | SyntaxKind::GetAccessor
                                        | SyntaxKind::SetAccessor
                                        | SyntaxKind::ClassDeclaration
                                        | SyntaxKind::ClassExpression
                                ) {
                                    break;
                                }
                                boundary = b.parent();
                            }
                            cur = landed.and_then(|b| b.parent());
                            continue 'walk;
                        }
                        _ => {}
                    }
                }
                if is_fn_like(&a) {
                    if immediately_invoked(&a) {
                        cur = a.parent();
                        continue;
                    }
                    exempt = true;
                    break;
                }

                if a.kind == SyntaxKind::PropertyDeclaration {
                    let in_initializer = matches!(&a.data, tsox_frontend::ast::NodeData::PropertyDeclaration(pd) if pd.initializer.as_ref().is_some_and(|init| init.loc.contains(node.loc.pos())));
                    let is_static_prop = a.has_syntactic_modifier(ModifierFlags::Static);
                    let is_decl_instance_prop = declaration_for_scope.kind
                        == SyntaxKind::PropertyDeclaration
                        && !declaration_for_scope.has_syntactic_modifier(ModifierFlags::Static);
                    if in_initializer && !is_static_prop && !is_decl_instance_prop {
                        exempt = true;
                        break;
                    }
                }
                cur = a.parent();
            }
            if exempt {
                return true;
            }
        }
        false
    }
}
