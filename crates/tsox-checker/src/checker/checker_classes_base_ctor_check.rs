#![allow(unused_imports)]

use crate::checker::checker::CheckMode;
use crate::checker::checker_classes::*;

// Go getBaseConstructorTypeOfClass 的合法性检查段：TS2507/TS2735 在
// get_base_constructor_type_of_class 内随首解析记忆化发射，此处仅触发解析
impl Checker {
    pub(crate) fn check_base_constructor_type(&mut self, expr: &Arc<Node>) {
        let Some(class_node) = expr
            .parent()
            .and_then(|ewa| ewa.parent())
            .and_then(|clause| clause.parent())
            .filter(|class| {
                matches!(
                    class.kind,
                    SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
                )
            })
        else {
            return;
        };
        let shell = self.build_class_instance_type_with_base(&class_node);
        let _ = self.get_base_constructor_type_of_class(&shell);
    }

    // Go checkExpression(heritage 表达式) 的值语义取型：标识符按 Value 含义
    // 解析到符号后取其值类型（类符号得构造器侧静态型）；Value 含义解析失败
    // 对应 Go resolveEntityName 失败得 errorType
    pub(crate) fn heritage_extends_value_type(&mut self, expr: &Arc<Node>) -> Arc<Type> {
        if expr.kind == SyntaxKind::Identifier {
            let Some(symbol) = self.resolve_identifier_with_meaning(
                expr,
                SymbolFlags::VALUE | SymbolFlags::ExportValue,
            ) else {
                return self.error_type();
            };
            if symbol.flags.intersects(SymbolFlags::Class) {
                if let Some(class_node) = symbol
                    .declarations
                    .iter()
                    .find(|d| d.kind == SyntaxKind::ClassDeclaration)
                    .cloned()
                {
                    let ctor_type = self.get_type_of_class_declaration(&class_node);
                    if !self
                        .get_signatures_of_type(&ctor_type, SignatureKind::Construct)
                        .is_empty()
                    {
                        return ctor_type;
                    }
                }
            }
            return self.get_type_of_symbol(&symbol);
        }
        self.check_expression_ex(expr, CheckMode::Normal)
    }

    // Go getBaseTypeVariableOfClass：基构造类型本身是类型变量（或含类型变量
    // 的交集）时返回之；实例壳经 class_instance_type_cache 稳定驻留，基构造
    // 类型随之复用首解析记忆化
    pub(crate) fn class_base_type_variable(
        &mut self,
        class_node: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        let shell = self.build_class_instance_type_with_base(class_node);
        let base_constructor_type = self.get_base_constructor_type_of_class(&shell)?;
        if base_constructor_type.flags.contains(TypeFlags::TypeParameter) {
            return Some(base_constructor_type);
        }
        if base_constructor_type.is_intersection() {
            return base_constructor_type
                .types()?
                .iter()
                .find(|t| t.flags.contains(TypeFlags::TypeParameter))
                .cloned();
        }
        None
    }
}

impl Checker {
    // Go checkClassLikeDeclaration 的 mixin 分支：基构造类型是类型变量时，
    // 自身静态型须为 mixin 构造型（isMixinConstructorType），否则 TS2545
    pub(crate) fn check_mixin_constructor_type(&mut self, class_node: &Arc<Node>) {
        let shell = self.build_class_instance_type_with_base(class_node);
        let Some(base_constructor_type) = self.get_base_constructor_type_of_class(&shell) else {
            return;
        };
        if !base_constructor_type.flags.contains(TypeFlags::TypeParameter) {
            return;
        }
        let static_type = self.get_type_of_class_declaration(class_node);
        if !self.is_mixin_constructor_type(&static_type) {
            let loc = class_node
                .name()
                .map(|n| n.loc)
                .unwrap_or(class_node.loc);
            let file = self.current_file.clone();
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                loc,
                tsox_core::diagnostics::messages_generated::
                    A_MIXIN_CLASS_MUST_HAVE_A_CONSTRUCTOR_WITH_A_SINGLE_REST_PARAMETER_OF_TYPE_ANY,
                vec![],
            ));
        }
    }

    // Go getDefaultConstructSignatures：无自有构造器时从基构造类型继承构造
    // 签名（类型变量经约束解析，交集聚合），返回类型改写为本类实例类型
    pub(crate) fn inherit_base_constructor_signatures(
        &mut self,
        class_node: &Arc<Node>,
        instance_type: &Arc<Type>,
        out: &mut Vec<Arc<Signature>>,
    ) {
        use crate::checker::checker_classes_ctor_super_calls::{
            class_extends_heritage_element, expression_with_type_arguments_expression,
        };
        use crate::checker::types::Signature;
        use std::sync::OnceLock;
        let Some(heritage_element) = class_extends_heritage_element(class_node) else {
            return;
        };
        let expr = expression_with_type_arguments_expression(&heritage_element);
        if expr.kind == SyntaxKind::NullKeyword {
            return;
        }
        let value_type = self.get_type_of_node(&expr);
        if value_type.flags.contains(TypeFlags::Any) || self.is_error_type(&value_type) {
            return;
        }
        let base_type = if value_type.is_type_parameter() {
            self.get_constraint_of_type_parameter(&value_type)
                .unwrap_or_else(|| Arc::clone(&value_type))
        } else {
            Arc::clone(&value_type)
        };
        let derived_is_abstract = class_node.has_syntactic_modifier(ModifierFlags::Abstract);
        let heritage_args: Vec<Arc<Type>> = match &heritage_element.data {
            NodeData::ExpressionWithTypeArguments(ewa) => ewa
                .type_arguments
                .as_ref()
                .map(|n| {
                    n.iter()
                        .map(|a| self.get_type_from_type_node(a))
                        .collect()
                })
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        let class_tps = self.class_type_parameter_types_of(class_node);
        for sig in self.get_signatures_of_type(&base_type, SignatureKind::Construct) {
            let type_param_count = sig.type_parameters.len();
            let min_type_arg_count = self.get_min_type_argument_count(&sig.type_parameters);
            if heritage_args.len() < min_type_arg_count || heritage_args.len() > type_param_count {
                continue;
            }
            let mut flags = sig.flags;
            if derived_is_abstract {
                flags |= SignatureFlags::Abstract;
            } else {
                flags.remove(SignatureFlags::Abstract);
            }
            let instantiated = if type_param_count != 0 {
                let filled = self.fill_missing_type_arguments(
                    &heritage_args,
                    &sig.type_parameters,
                    min_type_arg_count,
                    false,
                );
                self.get_signature_instantiation(&sig, &filled)
            } else {
                sig
            };
            let inherited = Signature {
                id: instantiated.id,
                flags,
                min_argument_count: instantiated.min_argument_count,
                resolved_min_argument_count: instantiated.resolved_min_argument_count,
                declaration: instantiated.declaration.clone(),
                type_parameters: class_tps.clone(),
                parameters: instantiated.parameters.clone(),
                this_parameter: instantiated.this_parameter.clone(),
                resolved_return_type: OnceLock::from(Arc::clone(instance_type)),
                resolved_type_predicate: instantiated.resolved_type_predicate.clone(),
                target: Some(Arc::clone(&instantiated)),
                mapper: instantiated.mapper.clone(),
                isolated_signature_type: OnceLock::new(),
                instantiated_parameter_types: instantiated.instantiated_parameter_types.clone(),
            };
            out.push(Arc::new(inherited));
        }
    }
}
