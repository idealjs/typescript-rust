//! Go checkIdentifier initialType switch（checker.go:11742-11777）的读引用
//! initial 判定，供 get_type_of_identifier / typeof 拉取链传入流查询。

use std::sync::Arc;
use tsox_frontend::ast::{FlowFlags, FlowNode, Node, NodeFlags, Symbol, SymbolFlags, SyntaxKind};
use crate::checker::checker::Checker;
use crate::checker::mig::m1b::variable_declaration_exclamation_token;
use crate::checker::types::Type;
use crate::checker::types_type_id::{TypeFlags, TYPE_FLAGS_ANY_OR_UNKNOWN};
use crate::checker::utilities_get_assignment_target::is_in_type_query;

impl Checker {
    // Go checkIdentifier（checker.go:11717-11777）assumeInitialized 全子句 +
    // initialType switch：读引用流查询的 initialType 仅在 typeIsAutomatic 且
    // !assumeInitialized 时为 undefinedType（assumeInitialized 时 initial=t，
    // 与 initial=declared 同值）。返回 None 表示 initial 与 declared 同型，
    // 查询维持现缓存路径。
    //
    // 门控：仅读引用的流节点即容器 FlowStart 时启用。流走查需穿过
    // loop/branch label 的读（循环体内/闭包于循环头），Go 经
    // checkExpressionCached（checker.go:7977-7996 flowLoopStack/flowTypeCache
    // 重置）+ getTypeAtFlowLoopLabel（flow.go:1374 flowLoopCache）的重启
    // 机器把演化 any 读落回 declared 呈 any（corpus 实测 capturedLetConstInLoop1
    // 等族 Go=any），该机器未移植；FlowStart 直挂读（顶层/纯块未赋值演化
    // any，如 assignmentToParenthesizedExpression1、nestedBlockScopedBindings2
    // 族）Go 渲染 initialType=undefinedType（Go 参考基线实测）。
    pub(crate) fn reference_read_initial_type_override(
        &mut self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
        t: &Arc<Type>,
        flow: Option<&Arc<FlowNode>>,
    ) -> Option<Arc<Type>> {
        if !flow.is_some_and(|f| f.flags.contains(FlowFlags::START)) {
            return None;
        }
        let declaration = symbol.value_declaration.clone()?;
        // Go getTypeForVariableLikeDeclaration（checker.go:17536-17768）有序分层：
        // for-in/of 头声明（17616-17645）、绑定元素、注解、初始化式先行返回，
        // auto 门（17686-17700）仅对无注解、无（非 null/undefined）初始化式、
        // 非绑定模式名、非 const、非导出、非 ambient、非 for-in/of 的普通
        // var/let 且 noImplicitAny 开启时可达——t=auto 必然源出此形状。声明
        // 供源若把 auto 单例挂到其他形状（绑定元素/迭代失败回退等，Go 该 t
        // 恒非 auto，switch 走 default 臂 getOptionalType(t)），此处不启用即
        // 对齐 Go 实效语义
        let is_auto_declared_shape = tsox_frontend::ast::is_variable_declaration(&declaration)
            && !declaration
                .parent()
                .and_then(|p| p.parent())
                .is_some_and(|gp| tsox_frontend::ast::is_for_in_or_of_statement(&gp))
            && declaration
                .initializer()
                .map(|init| self.is_null_or_undefined(&init))
                .unwrap_or(true)
            && !declaration
                .name()
                .as_ref()
                .is_some_and(|n| tsox_frontend::ast::node_data_generated::is_binding_pattern(n))
            && self.no_implicit_any
            && self
                .get_combined_modifier_flags_cached(&declaration)
                .intersection(tsox_frontend::ast::ModifierFlags::Export)
                .is_empty()
            && declaration
                .flags
                .intersection(NodeFlags::Ambient)
                .is_empty()
            && !self
                .get_combined_node_flags_cached(&declaration)
                .contains(NodeFlags::Constant);
        if !is_auto_declared_shape {
            return None;
        }
        let type_is_automatic =
            Arc::ptr_eq(t, &self.auto_type()) || Arc::ptr_eq(t, &self.auto_array_type());
        if !type_is_automatic {
            return None;
        }
        let is_parameter =
            tsox_frontend::ast::get_root_declaration(&declaration).kind == SyntaxKind::Parameter;
        let declaration_container = self.get_control_flow_container(&declaration);
        let flow_container = self.get_control_flow_container(node);
        let is_outer_variable = !Arc::ptr_eq(&flow_container, &declaration_container);
        let is_spread_destructuring_assignment_target = node
            .parent()
            .map(|parent| {
                parent
                    .parent()
                    .map(|grandparent| {
                        tsox_frontend::ast::is_spread_assignment(&parent)
                            && self.is_destructuring_assignment_target(&grandparent)
                    })
                    .unwrap_or(false)
            })
            .unwrap_or(false);
        let is_module_exports = symbol.flags.contains(SymbolFlags::ModuleExports);
        let is_never_initialized = tsox_frontend::ast::is_variable_declaration(&declaration)
            && !declaration
                .parent()
                .and_then(|p| p.parent())
                .map(|gp| tsox_frontend::ast::is_for_in_or_of_statement(&gp))
                .unwrap_or(false)
            && declaration.initializer().is_none()
            && variable_declaration_exclamation_token(&declaration).is_none()
            && self.is_mutable_local_variable_declaration(&declaration)
            && !self.is_symbol_assigned_definitely(symbol);
        let assume_initialized = is_parameter
            || symbol.flags.contains(SymbolFlags::Alias)
            || (is_outer_variable && !is_never_initialized)
            || is_spread_destructuring_assignment_target
            || is_module_exports
            || self.is_same_scoped_binding_element(node, &declaration)
            || (!Arc::ptr_eq(t, &self.auto_type())
                && !Arc::ptr_eq(t, &self.auto_array_type())
                && (!self.strict_null_checks
                    || t.flags.intersects(TYPE_FLAGS_ANY_OR_UNKNOWN | TypeFlags::Void)
                    || is_in_type_query(node)
                    || self.is_in_ambient_or_type_node(node)
                    || node
                        .parent()
                        .map(|p| p.kind == SyntaxKind::ExportSpecifier)
                        .unwrap_or(false)))
            || node
                .parent()
                .map(|p| tsox_frontend::ast::is_non_null_expression(&p))
                .unwrap_or(false)
            || (tsox_frontend::ast::is_variable_declaration(&declaration)
                && variable_declaration_exclamation_token(&declaration).is_some())
            || declaration.flags.contains(NodeFlags::Ambient);
        if assume_initialized {
            None
        } else {
            Some(self.undefined_type())
        }
    }

    // Go checkIdentifier 的 initialType 参数臂（checker.go:11732 声明根为
    // Parameter、:11758-11766 assumeInitialized 首析取支 isParameter、
    // :11821-11828 removeOptionalityFromDeclaredType）：参数读引用的流初始型
    // 对带初始值参数剥 undefined，与检查链 check_identifier（mig/m1b_2.rs:258）
    // 同款同门——removeOptionalityFromDeclaredType 内部门（strictNullChecks、
    // 参数声明、有初始化式、hasTypeFacts(IS_UNDEFINED)、初始化式不含
    // undefined）不命中时原样返回。返回 None = initial 与 declared 同型，
    // 查询维持现缓存路径
    pub(crate) fn check_identifier_parameter_initial_type(
        &mut self,
        symbol: &Arc<Symbol>,
        t: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let declaration = symbol.value_declaration.clone()?;
        if tsox_frontend::ast::get_root_declaration(&declaration).kind != SyntaxKind::Parameter {
            return None;
        }
        let stripped = self.remove_optionality_from_declared_type(t, &declaration);
        if Arc::ptr_eq(&stripped, t) {
            None
        } else {
            Some(stripped)
        }
    }
}
