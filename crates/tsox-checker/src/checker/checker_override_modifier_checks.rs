#![allow(unused_imports)]

use std::sync::Arc;

use tsox_frontend::ast::{Node, NodeData, SyntaxKind};

use crate::checker::checker::*;
use crate::checker::types::TypeFlags;

impl Checker {
    /// Go checkMembersForOverrideModifier：类成员（含构造器参数属性）的
    /// override 修饰符检查（TS4112/4113/4114/4115/4116）
    pub(crate) fn check_members_for_override_modifier(&mut self, class_node: &Arc<Node>) {
        let members = match &class_node.data {
            NodeData::ClassDeclaration(d) => Arc::clone(&d.members),
            NodeData::ClassExpression(d) => Arc::clone(&d.members),
            _ => return,
        };
        let base = self.override_base_class_node(class_node);
        let base_instance = base.as_ref().map(|b| self.build_class_instance_type_with_base(b));
        let base_static = base.as_ref().map(|b| self.get_type_of_class_declaration(b));
        let instance = self.build_class_instance_type_with_base(class_node);
        let static_type = self.get_type_of_class_declaration(class_node);
        let class_non_ambient = !class_node.has_syntactic_modifier(ModifierFlags::Ambient)
            && self.ambient_context_depth == 0
            && !self
                .current_file
                .as_ref()
                .is_some_and(|f| f.is_declaration_file);

        for member in members.iter() {
            if member.has_syntactic_modifier(ModifierFlags::Ambient) {
                continue;
            }
            if member.kind == SyntaxKind::Constructor {
                let NodeData::ConstructorDeclaration(cd) = &member.data else {
                    continue;
                };
                for param in cd.parameters.iter() {
                    let NodeData::ParameterDeclaration(pd) = &param.data else {
                        continue;
                    };
                    let has_ctor_prop_mods = pd
                        .modifiers
                        .as_ref()
                        .is_some_and(|m| {
                            m.modifier_flags.intersects(
                                ModifierFlags::Public
                                    | ModifierFlags::Private
                                    | ModifierFlags::Protected
                                    | ModifierFlags::Readonly,
                            )
                        });
                    if has_ctor_prop_mods {
                        self.check_member_for_override_modifier(
                            class_node,
                            param,
                            true,
                            &instance,
                            &static_type,
                            base_instance.as_ref(),
                            base_static.as_ref(),
                            class_non_ambient,
                        );
                    }
                }
                continue;
            }
            if member.kind == SyntaxKind::ClassStaticBlockDeclaration
                || member.kind == SyntaxKind::IndexSignature
            {
                continue;
            }
            self.check_member_for_override_modifier(
                class_node,
                member,
                false,
                &instance,
                &static_type,
                base_instance.as_ref(),
                base_static.as_ref(),
                class_non_ambient,
            );
        }
    }

    fn check_member_for_override_modifier(
        &mut self,
        class_node: &Arc<Node>,
        member: &Arc<Node>,
        is_param_property: bool,
        instance: &Arc<crate::checker::types::Type>,
        static_type: &Arc<crate::checker::types::Type>,
        base_instance: Option<&Arc<crate::checker::types::Type>>,
        base_static: Option<&Arc<crate::checker::types::Type>>,
        class_non_ambient: bool,
    ) {
        use tsox_core::diagnostics::messages_generated as msg;
        let Some(name_node) = member.name() else {
            return;
        };
        // Go GetErrorRangeForNode：Parameter 用节点自身范围（含修饰符），
        // 其余成员用名字节点
        let error_anchor = if is_param_property {
            member.loc
        } else {
            name_node.loc
        };
        let has_override = member.has_syntactic_modifier(ModifierFlags::Override);
        let has_abstract = member.has_syntactic_modifier(ModifierFlags::Abstract);
        let is_static = member.has_syntactic_modifier(ModifierFlags::Static);

        // 动态名（非字面量、非 late-bindable 计算名）不可带 override
        if has_override && name_node.kind == SyntaxKind::ComputedPropertyName {
            let bindable = match &name_node.data {
                NodeData::ComputedPropertyName(cd) => {
                    crate::binder::symbols_binder_4::well_known_symbol_member_name(&cd.expression)
                        .is_some()
                        || matches!(
                            cd.expression.kind,
                            SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral
                        )
                }
                _ => false,
            };
            if !bindable {
                self.emit_override_error(
                    &error_anchor,
                    msg::THIS_MEMBER_CANNOT_HAVE_AN_OVERRIDE_MODIFIER_BECAUSE_ITS_NAME_IS_DYNAMIC,
                    vec![],
                );
                return;
            }
        }

        let key = match name_node.kind {
            SyntaxKind::Identifier
            | SyntaxKind::StringLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::PrivateIdentifier => name_node.text().to_string(),
            SyntaxKind::ComputedPropertyName => match &name_node.data {
                NodeData::ComputedPropertyName(cd) => {
                    match crate::binder::symbols_binder_4::well_known_symbol_member_name(
                        &cd.expression,
                    ) {
                        Some(internal) => internal,
                        None => return,
                    }
                }
                _ => return,
            },
            _ => return,
        };

        let no_implicit_override = self.compiler_options.no_implicit_override.is_true();

        if let (Some(base_inst), Some(base_stat)) = (base_instance, base_static) {
            let base_display = self
                .override_base_class_node(class_node)
                .map(|b| Self::class_display_name(&b))
                .unwrap_or_default();
            if has_override || no_implicit_override {
                let this_type = if is_static {
                    Arc::clone(static_type)
                } else {
                    Arc::clone(instance)
                };
                let base_type = if is_static {
                    Arc::clone(base_stat)
                } else {
                    Arc::clone(base_inst)
                };
                let prop = self.get_property_of_type(&this_type, &key);
                let base_prop = self.get_property_of_type(&base_type, &key);

                if prop.is_some() && base_prop.is_none() && has_override {
                    let base_str = base_display.clone();
                    self.emit_override_error(
                        &error_anchor,
                        msg::THIS_MEMBER_CANNOT_HAVE_AN_OVERRIDE_MODIFIER_BECAUSE_IT_IS_NOT_DECLARED_IN_THE_BASE_CLASS_0,
                        vec![base_str],
                    );
                    return;
                }

                if let (Some(_), Some(bp)) = (&prop, &base_prop) {
                    if !bp.declarations.is_empty()
                        && no_implicit_override
                        && class_non_ambient
                    {
                        if has_override {
                            return;
                        }
                        let base_has_abstract = bp
                            .declarations
                            .iter()
                            .any(|d| d.has_syntactic_modifier(ModifierFlags::Abstract));
                        let base_str = base_display.clone();
                        if !base_has_abstract {
                            let message = if is_param_property {
                                msg::THIS_PARAMETER_PROPERTY_MUST_HAVE_AN_OVERRIDE_MODIFIER_BECAUSE_IT_OVERRIDES_A_MEMBER_IN_BASE_CLASS_0
                            } else {
                                msg::THIS_MEMBER_MUST_HAVE_AN_OVERRIDE_MODIFIER_BECAUSE_IT_OVERRIDES_A_MEMBER_IN_THE_BASE_CLASS_0
                            };
                            self.emit_override_error(&error_anchor, message, vec![base_str]);
                        } else if has_abstract {
                            self.emit_override_error(
                                &error_anchor,
                                msg::THIS_MEMBER_MUST_HAVE_AN_OVERRIDE_MODIFIER_BECAUSE_IT_OVERRIDES_AN_ABSTRACT_METHOD_THAT_IS_DECLARED_IN_THE_BASE_CLASS_0,
                                vec![base_str],
                            );
                        }
                    }
                }
            }
        } else if has_override {
            let class_str = Self::class_display_name(class_node);
            self.emit_override_error(
                &error_anchor,
                msg::THIS_MEMBER_CANNOT_HAVE_AN_OVERRIDE_MODIFIER_BECAUSE_ITS_CONTAINING_CLASS_0_DOES_NOT_EXTEND_ANOTHER_CLASS,
                vec![class_str],
            );
        }
    }

    /// extends 基类节点：Identifier 经 extends_base_of 解析；
    /// 类表达式基类直接用该节点
    fn override_base_class_node(&self, class_node: &Arc<Node>) -> Option<Arc<Node>> {
        if let Some((b, _)) = self.extends_base_of(class_node) {
            return Some(b);
        }
        let heritage = match &class_node.data {
            NodeData::ClassDeclaration(d) => d.heritage_clauses.as_ref(),
            NodeData::ClassExpression(d) => d.heritage_clauses.as_ref(),
            _ => None,
        }?;
        for clause in heritage.iter() {
            if let NodeData::HeritageClause(hc) = &clause.data
                && hc.token == SyntaxKind::ExtendsKeyword
                && let Some(first) = hc.types.iter().next()
                && let NodeData::ExpressionWithTypeArguments(ewa) = &first.data
            {
                let mut base_expr = &ewa.expression;
                while let NodeData::ParenthesizedExpression(pe) = &base_expr.data {
                    base_expr = &pe.expression;
                }
                if base_expr.kind == SyntaxKind::ClassExpression {
                    return Some(Arc::clone(base_expr));
                }
            }
        }
        None
    }

    fn class_display_name(class_node: &Arc<Node>) -> String {
        match &class_node.data {
            NodeData::ClassDeclaration(d) => d
                .name
                .as_ref()
                .map(|n| n.text().to_string())
                .unwrap_or_else(|| "(Anonymous class)".to_string()),
            NodeData::ClassExpression(d) => d
                .name
                .as_ref()
                .map(|n| n.text().to_string())
                .unwrap_or_else(|| "(Anonymous class)".to_string()),
            _ => "(Anonymous class)".to_string(),
        }
    }

    fn emit_override_error(
        &mut self,
        loc: &tsox_core::core::text::TextRange,
        message: tsox_core::diagnostics::Message,
        args: Vec<String>,
    ) {
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            self.current_file.clone(),
            *loc,
            message,
            args,
        ));
    }
}

#[allow(dead_code)]
fn unused_typeflags_marker(_: TypeFlags) {}
