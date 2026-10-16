#![allow(unused_imports)]

use crate::checker::nodebuilder::*;
use std::sync::Arc;
use crate::checker::types_type_id::TypeFlags;
use tsox_frontend::ast::{get_name_of_declaration, Node, Symbol, SymbolFlags};

impl Checker {
    /// Go getPropertyNameNodeForSymbolFromNameType（nodebuilderimpl.go:2611-2659）
    /// 枚举字面量名型分支：late-bound 计算名属性的 nameType 为枚举成员字面量型
    /// 时，属性名重建为 [枚举.成员] 计算名（NewComputedPropertyName(
    /// symbolToExpression(nameType.symbol, Value))）。门为枚举符号对
    /// enclosingDeclaration 的值可及性（nodebuilderimpl.go:2620-2630）；
    /// enclosing 缺省时 Go 同样回退 enclosingFile，双双为空（错误消息通道
    /// enclosingDeclaration=nil）则门失败，落字面量名。
    pub(crate) fn get_property_name_node_for_symbol_from_name_type(
        &mut self,
        prop: &Arc<Symbol>,
    ) -> Option<String> {
        ::tsox_core::fntrace::enter("get_property_name_node_for_symbol_from_name_type");
        let name_type = self
            .value_symbol_links
            .get(prop)
            .and_then(|l| l.name_type.clone())?;
        if !name_type.flags.intersects(TypeFlags::EnumLiteral) {
            return None;
        }
        let member_sym = name_type.symbol.clone()?;
        let enum_sym = member_sym
            .parent()
            .unwrap_or_else(|| Arc::clone(&member_sym));
        let enclosing = self.type_render_enclosing.clone()?;
        if !self.is_symbol_accessible_by_flags(&enum_sym, Some(&enclosing), SymbolFlags::VALUE) {
            return None;
        }
        let parent_name = self
            .accessible_chain_full_name(&enum_sym, SymbolFlags::VALUE)
            .unwrap_or_else(|| self.namespace_qualified_name(&enum_sym, SymbolFlags::VALUE));
        // 成员段按书写名取自符号链（createExpressionFromSymbolChain
        // nodebuilderimpl.go:887-939）：标识符名走属性访问（E1.A），其余按
        // 元素访问（E1["name"]，书写名原样入括、已带方括号者剥一层）
        let member_written = member_sym
            .declarations
            .first()
            .and_then(|d| get_name_of_declaration(d))
            .map(|n| tsox_frontend::scanner::mig::m3i::declaration_name_to_string(Some(&n)))
            .unwrap_or_else(|| member_sym.name.clone());
        if crate::checker::checker_get_excluded_symbol_flags::is_valid_identifier_text(
            &member_written,
        ) {
            Some(format!("[{}.{}]", parent_name, member_written))
        } else {
            let mut inner = member_written.as_str();
            if inner.starts_with('[') && inner.ends_with(']') && inner.len() >= 2 {
                inner = &inner[1..inner.len() - 1];
            }
            Some(format!("[{}[{}]]", parent_name, inner))
        }
    }
}
