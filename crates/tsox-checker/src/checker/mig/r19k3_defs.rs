#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::types_impl_chunk::LiteralValue;
use std::sync::{Arc, OnceLock};
use tsox_frontend::ast::node_data_generated::{
    ArrayLiteralExpressionData, BinaryExpressionData, BindingPatternData,
    ClassStaticBlockDeclarationData, ConstructorDeclarationData, JsxTextData, PrefixUnaryExpressionData,
    ShorthandPropertyAssignmentData, SourceFileData, TypeLiteralNodeData,
};
use tsox_frontend::ast::mig::m3e::FlowSwitchClauseData;
use tsox_frontend::ast::{Node, NodeData, SyntaxKind};
use tsox_frontend::ast::{is_string_or_numeric_literal_like};

use crate::checker::types_type_id::TYPE_FLAGS_NULLABLE;

pub trait NodeAccessExtR19k3 {
    fn as_flow_switch_clause_data(&self) -> &FlowSwitchClauseData;
    fn as_source_file(&self) -> &SourceFileData;
    fn as_type_literal_node(&self) -> &TypeLiteralNodeData;
    fn as_prefix_unary_expression(&self) -> &PrefixUnaryExpressionData;
    fn as_binary_expression_node(&self) -> &BinaryExpressionData;
    fn as_array_literal_expression_node(&self) -> &ArrayLiteralExpressionData;
    fn as_shorthand_property_assignment_node(&self) -> &ShorthandPropertyAssignmentData;
    fn as_binding_pattern_node(&self) -> &BindingPatternData;
    fn as_constructor_declaration(&self) -> &ConstructorDeclarationData;
    fn as_class_static_block_declaration(&self) -> &ClassStaticBlockDeclarationData;
    fn as_jsx_text(&self) -> &JsxTextData;
    fn as_js_doc(&self) -> &tsox_frontend::ast::node_data_generated::JSDocData;
}

macro_rules! r19k3_as_data {
    ($name:ident, $variant:ident, $data:ty) => {
        fn $name(&self) -> &$data {
            match &self.data {
                NodeData::$variant(d) => d,
                _ => panic!("unexpected node data: {:?}", self.kind),
            }
        }
    };
}

impl NodeAccessExtR19k3 for Node {
    r19k3_as_data!(as_flow_switch_clause_data, FlowSwitchClauseData, FlowSwitchClauseData);
    r19k3_as_data!(as_source_file, SourceFile, SourceFileData);
    r19k3_as_data!(as_type_literal_node, TypeLiteralNode, TypeLiteralNodeData);
    r19k3_as_data!(as_prefix_unary_expression, PrefixUnaryExpression, PrefixUnaryExpressionData);
    r19k3_as_data!(as_binary_expression_node, BinaryExpression, BinaryExpressionData);
    r19k3_as_data!(
        as_array_literal_expression_node,
        ArrayLiteralExpression,
        ArrayLiteralExpressionData
    );
    r19k3_as_data!(
        as_shorthand_property_assignment_node,
        ShorthandPropertyAssignment,
        ShorthandPropertyAssignmentData
    );
    r19k3_as_data!(as_binding_pattern_node, BindingPattern, BindingPatternData);
    r19k3_as_data!(
        as_constructor_declaration,
        ConstructorDeclaration,
        ConstructorDeclarationData
    );
    r19k3_as_data!(
        as_class_static_block_declaration,
        ClassStaticBlockDeclaration,
        ClassStaticBlockDeclarationData
    );
    r19k3_as_data!(as_jsx_text, JsxText, JsxTextData);
    r19k3_as_data!(as_js_doc, JSDoc, tsox_frontend::ast::node_data_generated::JSDocData);
}

pub(crate) fn any_to_string(v: &LiteralValue) -> String { ::tsox_core::fntrace::enter("any_to_string"); 
    match v {
        LiteralValue::String(s) => s.clone(),
        LiteralValue::Number(n) => n.to_string(),
        LiteralValue::BigInt(b) => b.to_string(),
        LiteralValue::Boolean(b) => b.to_string(),
        LiteralValue::None => String::new(),
    }
}

pub(crate) fn has_only_expression_initializer(node: &Node) -> bool { ::tsox_core::fntrace::enter("has_only_expression_initializer"); 
    matches!(
        node.kind,
        SyntaxKind::VariableDeclaration
            | SyntaxKind::Parameter
            | SyntaxKind::BindingElement
            | SyntaxKind::PropertyAssignment
            | SyntaxKind::EnumMember
            | SyntaxKind::JsxAttribute
            | SyntaxKind::ShorthandPropertyAssignment
    )
}

pub(crate) fn try_get_text_of_property_name(name: &Arc<Node>) -> Option<String> { ::tsox_core::fntrace::enter("try_get_text_of_property_name"); 
    match name.kind {
        SyntaxKind::Identifier
        | SyntaxKind::PrivateIdentifier
        | SyntaxKind::StringLiteral
        | SyntaxKind::NumericLiteral => Some(name.text().to_string()),
        SyntaxKind::ComputedPropertyName => {
            let expr = name.expression()?;
            if is_string_or_numeric_literal_like(expr) {
                Some(expr.text().to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

pub(crate) fn get_candidate_variable_declaration_initializer(
    declaration: Option<&Arc<Node>>,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_candidate_variable_declaration_initializer"); 
    let declaration = declaration?;
    let parent = declaration.parent()?;
    if parent.kind != SyntaxKind::VariableDeclarationList {
        return None;
    }
    declaration.initializer().cloned()
}

pub(crate) fn non_dotted_name_cache_key() -> crate::checker::types_cached_type_kind::CacheHashKey { ::tsox_core::fntrace::enter("non_dotted_name_cache_key"); 
    static KEY: OnceLock<crate::checker::types_cached_type_kind::CacheHashKey> = OnceLock::new();
    *KEY.get_or_init(|| crate::checker::types_cached_type_kind::CacheHashKey::new(u64::MAX, u64::MAX))
}

impl Checker {
    pub(crate) fn get_type_of_expression(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_expression"); 
        // Go getTypeOfExpression（checker.go:7785-7809）：先查已解析缓存再重查
        //（flowTypeCache[node] 命中即返，:7793）。.types 拉取查询对属性访问
        // name 的宿主上提（getRegularTypeOfExpression → getTypeOfExpression）
        // 依赖该缓存返回 check 阶段已解析的宿主表达式型；无缓存裸重查会把
        // 成员回落到声明型（v1.x 的 x 渲染 T 而非 string）。本侧缓存落
        // type_node_links（check_expression_cached_ex 驻留点 m1b.rs:244），
        // 与 get_type_of_node 入口同源。重查按 CheckModeTypeOnly
        //（checker.go:7797，拉取式求值免诊断）
        // 例外：发射侧（types_emit_query）数组字面量零缓存读——Go
        // getTypeOfNode（checker.go:33776-33876）入口不读 typeNodeLinks，
        // check 期驻留的手写取型值（脱鲜）不经本入口回放压掉保鲜重查
        //（arrayCast ({ foo: string; } | {})[]），每次重查 TypeOnly 全链
        if !(self.types_emit_query && node.kind == SyntaxKind::ArrayLiteralExpression)
            && let Some(cached) = self
                .type_node_links
                .get(node)
                .and_then(|l| l.resolved_type.clone())
        {
            return cached;
        }
        self.check_expression_ex(node, CheckMode::TypeOnly)
    }

    pub(crate) fn narrow_type_by_discriminant(
        &mut self,
        t: &Arc<Type>,
        access: &Arc<Node>,
        narrow_type: &dyn Fn(&mut Checker, &Arc<Type>) -> Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_type_by_discriminant"); 
        let Some(prop_name) = self.get_accessed_property_name(access) else {
            return Arc::clone(t);
        };
        if prop_name.is_empty() {
            return Arc::clone(t);
        }
        let optional_chain = tsox_frontend::ast::is_optional_chain(access);
        let remove_nullable = self.strict_null_checks
            && (optional_chain
                || crate::checker::utilities_has_only_expression_initialization::is_non_null_access(
                    access,
                ))
            && self.maybe_type_of_kind(t, TYPE_FLAGS_NULLABLE);
        let non_null_type = if remove_nullable {
            self.get_type_with_facts(t, TypeFacts::NE_UNDEFINED_OR_NULL)
        } else {
            Arc::clone(t)
        };
        let Some(prop_type) = self.get_type_of_property_of_type(&non_null_type, &prop_name) else {
            return Arc::clone(t);
        };
        let mut prop_type = prop_type;
        if remove_nullable && optional_chain {
            prop_type = self.get_optional_type(prop_type, false);
        }
        let narrowed_prop_type = narrow_type(self, &prop_type);
        crate::checker::mig::m2b::r22k6_defs::filter_type_ext(self, t, &mut |checker, t| {
            let discriminant_type = checker
                .get_type_of_property_or_index_signature_of_type(t, &prop_name)
                .unwrap_or_else(|| checker.unknown_type());
            !discriminant_type.flags.contains(TypeFlags::Never)
                && !narrowed_prop_type.flags.contains(TypeFlags::Never)
                && checker.are_types_comparable(&narrowed_prop_type, &discriminant_type)
        })
    }
}

impl Checker {
    pub(crate) fn get_type_with_default_opt(
        &mut self,
        t: &Arc<Type>,
        initializer: Option<&Arc<Node>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_with_default_opt"); 
        if let Some(initializer) = initializer {
            let non_undefined = self.get_non_undefined_type(t);
            let default_type = self.get_type_of_expression(initializer);
            return self.get_union_type(vec![non_undefined, default_type]);
        }
        Arc::clone(t)
    }

    pub(crate) fn create_final_array_type(&mut self, element_type: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("create_final_array_type"); 
        if element_type.flags.contains(TypeFlags::Never) {
            return self.auto_array_type();
        }
        if element_type.flags.contains(TypeFlags::Union) {
            let types = element_type
                .types()
                .map(|ts| ts.to_vec())
                .unwrap_or_default();
            let union_type = self.get_union_type(types);
            return self.create_array_type(union_type);
        }
        self.create_array_type(Arc::clone(element_type))
    }

    pub(crate) fn get_type_of_initializer(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_initializer"); 
        if let Some(t) = self.type_node_links.get(node).and_then(|l| l.resolved_type.clone()) {
            return t;
        }
        self.get_type_of_expression(node)
    }
}

thread_local! {
    static RESOLVING_EXPLICIT_TYPE_OF_ADD: std::cell::RefCell<std::collections::HashSet<usize>> =
        std::cell::RefCell::new(std::collections::HashSet::new());
    static RESOLVING_EXPLICIT_TYPE_OF_REMOVE: std::cell::RefCell<std::collections::HashSet<usize>> =
        std::cell::RefCell::new(std::collections::HashSet::new());
}

pub fn reset_resolving_explicit_type_of_symbol() {
    RESOLVING_EXPLICIT_TYPE_OF_ADD.with(|r| r.borrow_mut().clear());
    RESOLVING_EXPLICIT_TYPE_OF_REMOVE.with(|r| r.borrow_mut().clear());
}

pub(crate) fn resolving_explicit_type_of_symbol_add_if_absent(symbol: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("resolving_explicit_type_of_symbol_add_if_absent"); 
    RESOLVING_EXPLICIT_TYPE_OF_ADD.with(|r| {
        let mut r = r.borrow_mut();
        let key = Arc::as_ptr(symbol) as *const () as usize;
        r.insert(key)
    })
}

pub(crate) fn resolving_explicit_type_of_symbol_remove(symbol: &Arc<Symbol>) { ::tsox_core::fntrace::enter("resolving_explicit_type_of_symbol_remove"); 
    RESOLVING_EXPLICIT_TYPE_OF_REMOVE.with(|r| {
        let key = Arc::as_ptr(symbol) as *const () as usize;
        r.borrow_mut().remove(&key);
    })
}

impl Checker {
    pub(crate) fn report_diagnostic(
        &mut self,
        diag: &tsox_frontend::ast::Diagnostic,
        output: Option<&mut Vec<tsox_frontend::ast::Diagnostic>>,
    ) { ::tsox_core::fntrace::enter("report_diagnostic"); 
        if let Some(output) = output {
            output.push(diag.clone());
        } else {
            self.diagnostics.add(diag.clone());
        }
    }
}

pub(crate) fn has_type_parameter_default(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("has_type_parameter_default"); 
    use tsox_frontend::ast::node_data_generated::is_type_parameter_declaration;
    let Some(symbol) = &t.symbol else {
        return false;
    };
    symbol.declarations.iter().any(|d| {
        is_type_parameter_declaration(d)
            && match &d.data {
                NodeData::TypeParameterDeclaration(tp) => tp.default_type.is_some(),
                _ => false,
            }
    })
}
