use std::sync::Arc;

use tsox_frontend::ast::{Node, Symbol, SymbolFlags, SyntaxKind};

use crate::checker::checker::Checker;
use crate::checker::types_alias_symbol_links::SymbolAccessibility;

impl Checker {
    fn symbol_is_unknown(&self, symbol: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("symbol_is_unknown");
        self.unknown_symbol
            .as_ref()
            .is_some_and(|u| Arc::ptr_eq(u, symbol))
    }

    /// Go trackExistingEntityName（nodecopy.go:371-425，tryVisitTypeQuery
    /// nodecopy.go:427-443 的复用前置对账）：实体名在注解位置
    ///（resolveEntityName location=nil）与 enclosingDeclaration 两处解析，
    /// enclosing 处解到而不可及（函数容器起点 lastLocation==nil 豁免使命名
    /// 空间局部 var 命中，nameresolver.go:70-91）或两处引用不一致即
    /// introducesError；随后 serializeTypeName（nodebuilderimpl.go:457-475）
    /// 在注解位置复解（dontResolveAlias=false）仍解不到或不可及时整体复用
    /// 失败，回退 checker 序列化（reuseTypeNode nodecopy.go:87-101 的
    /// typeToTypeNode 兜底）
    pub(crate) fn track_existing_entity_name_introduces_error(
        &mut self,
        entity_name: &Arc<Node>,
    ) -> bool { ::tsox_core::fntrace::enter("track_existing_entity_name_introduces_error");
        // Go getMeaningOfEntityNameReference（emitresolver.go:325-332）：typeof
        // 实体名按 Value|ExportValue 位
        let meaning = SymbolFlags::VALUE | SymbolFlags::ExportValue;
        let leftmost = tsox_frontend::ast::mig::m3e_4::get_first_identifier(entity_name);
        let enclosing = self
            .type_render_enclosing
            .clone()
            .or_else(|| self.display_enclosing_node.clone());
        let mut introduces_error = false;
        let mut sym = self.resolve_entity_name(&leftmost, meaning, true, true, None);
        if enclosing.is_some()
            && !sym
                .as_ref()
                .is_some_and(|s| s.flags.contains(SymbolFlags::TypeParameter))
        {
            let exported = sym
                .as_ref()
                .map(|s| self.get_export_symbol_of_value_symbol_if_exported(s));
            let sym_at_location =
                self.resolve_entity_name(&leftmost, meaning, true, true, enclosing.as_ref());
            let mismatch = match (&sym_at_location, &exported) {
                (Some(at), Some(base)) => self.get_symbol_if_same_reference(at, base).is_none(),
                (None, Some(_)) => true,
                _ => false,
            };
            if sym_at_location.as_ref().is_some_and(|s| self.symbol_is_unknown(s)) || mismatch {
                introduces_error = true;
            } else {
                sym = sym_at_location;
            }
        }
        if !introduces_error
            && let Some(sym) = sym
        {
            // Go trackExistingEntityName：值声明为参数的函数域变量随参可及，
            // 免可及性判定
            let param_decl = sym.flags.contains(SymbolFlags::FunctionScopedVariable)
                && sym.value_declaration.as_ref().is_some_and(|d| {
                    tsox_frontend::ast::mig::m3g_2::is_part_of_parameter_declaration(d)
                        || d.kind == SyntaxKind::JSDocParameterTag
                });
            if !param_decl
                && !sym.flags.contains(SymbolFlags::TypeParameter)
                && !crate::checker::checker_get_excluded_symbol_flags::is_declaration_name(
                    entity_name,
                )
                && !self.go_is_symbol_accessible_for_reuse_gate(
                    &sym,
                    enclosing.as_ref(),
                    meaning,
                )
            {
                introduces_error = true;
            }
        }
        if !introduces_error {
            return false;
        }
        match self.resolve_entity_name(entity_name, SymbolFlags::VALUE, true, false, None) {
            Some(s) => {
                !self.go_is_symbol_accessible_for_reuse_gate(&s, enclosing.as_ref(), SymbolFlags::VALUE)
            }
            None => true,
        }
    }

    /// Go getContainersOfSymbol 候选臂（symbolaccessibility.go:305-339）：声明
    /// 宿主为非全局增强外部模块、模块块 export= 同名容器、类表达式 expando
    /// 赋值才产生容器候选；普通体局部声明全不命中
    fn go_container_candidates_exist(&self, symbol: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("go_container_candidates_exist"); 
        symbol.declarations.iter().any(|d| {
            d.kind == SyntaxKind::ClassExpression
                || d.parent().is_some_and(|p| {
                    crate::checker::symbolaccessibility_symbol_table_id::
                        has_non_global_augmentation_external_module_symbol(&p)
                        || p.kind == SyntaxKind::ModuleBlock
                })
        })
    }

    /// Go IsSymbolAccessible（symbolaccessibility.go:79-118 worker +
    /// :17-76 isAnySymbolAccessible）复判。本仓 get_containers_of_symbol
    /// （symbolaccessibility_checker_2.rs:261）为显示限定链补了函数体 Block
    /// 透明穿越臂（宿主函数符号作容器，baseExpressionTypeParameters 口径），
    /// 可及性路径借道该臂时，全局脚本顶层函数对 enclosingDeclaration 恒
    /// is_declaration_visible（函数声明宿主为 SourceFile 的全局文件），体局部
    /// var 被误判可及。Go getContainersOfSymbol（symbolaccessibility.go:293-347）
    /// 的 container 取 getParentOfSymbol（symbol.Parent），体局部符号 Parent 为
    /// nil（binder.go:1296-1301 declareSymbol parent=nil 透传），候选臂对体局部
    /// 声明全不命中，恒返回 nil——判定只余符号链臂：链非空且
    /// hasVisibleDeclarations 出结果即为终判，否则不可及（体局部符号无模块
    /// 逃逸，worker 兜底不可及）
    fn go_is_symbol_accessible_for_reuse_gate(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing: Option<&Arc<Node>>,
        meaning: SymbolFlags,
    ) -> bool { ::tsox_core::fntrace::enter("go_is_symbol_accessible_for_reuse_gate"); 
        if self.get_parent_of_symbol(symbol).is_some() || self.go_container_candidates_exist(symbol)
        {
            return self
                .is_symbol_accessible(symbol, enclosing, meaning, false)
                .accessibility
                == SymbolAccessibility::Accessible;
        }
        let chain = self.get_accessible_symbol_chain(symbol, enclosing, meaning, false);
        if let Some(head) = chain.first()
            && let Some(result) = self.has_visible_declarations_with_aliases(head, false)
        {
            return result.accessibility == SymbolAccessibility::Accessible;
        }
        false
    }
}
