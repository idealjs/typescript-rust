#![allow(unused_imports)]

use crate::checker::nodebuilder::*;
use crate::checker::types::*;

impl Checker {
    /// 属性访问接收者的字符串索引签名值类型（无具名成员时）
    pub(crate) fn receiver_string_index_type(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        use tsox_frontend::ast::SyntaxKind;
        let expression = match node.kind {
            SyntaxKind::PropertyAccessExpression => {
                let tsox_frontend::ast::NodeData::PropertyAccessExpression(pae) = &node.data
                else {
                    return None;
                };
                Arc::clone(&pae.expression)
            }
            _ => {
                let parent = node.parent.as_ref()?;
                if parent.kind != SyntaxKind::PropertyAccessExpression {
                    return None;
                }
                let tsox_frontend::ast::NodeData::PropertyAccessExpression(pae) = &parent.data
                else {
                    return None;
                };
                if !Arc::ptr_eq(&pae.name, node) {
                    return None;
                }
                Arc::clone(&pae.expression)
            }
        };
        let obj_type = self.get_type_of_node(&expression);
        let name = match node.kind {
            SyntaxKind::PropertyAccessExpression => {
                let tsox_frontend::ast::NodeData::PropertyAccessExpression(pae) = &node.data
                else {
                    return None;
                };
                pae.name.text()
            }
            _ => node.text(),
        };
        if self.get_property_of_type(&obj_type, name).is_some() {
            return None;
        }
        let structured = obj_type.as_structured()?;
        structured
            .index_infos
            .iter()
            .find(|info| {
                info.key_type
                    .as_ref()
                    .is_some_and(|k| k.flags.contains(TypeFlags::String))
            })
            .and_then(|info| info.value_type.clone())
    }

    /// 属性名命中接收者的字符串索引签名（无具名成员）：显示索引值类型（无前缀）。
    /// 兼容两种命中节点：名字标识符与整个属性访问表达式
    pub(crate) fn index_signature_property_parts(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Vec<SymbolDisplayPart>> {
        self.receiver_string_index_type(node)
            .map(|value| self.type_to_display_parts(&value))
    }

    /// Go shouldUsePlaceholderForProperty：反向映射属性的省略条件
    /// （1）已递归进入过的属性（2）嵌在非匿名源的反向映射内（3）深层同源 mapped
    pub(crate) fn should_use_placeholder_for_property(&self, prop: &Arc<Symbol>) -> bool {
        let result = self.should_use_placeholder_for_property_inner(prop);
                result
    }

    fn should_use_placeholder_for_property_inner(&self, prop: &Arc<Symbol>) -> bool {
        if !prop
            .check_flags
            .contains(tsox_frontend::ast::CheckFlags::ReverseMapped)
        {
            return false;
        }
        if self
            .reverse_mapped_print_stack
            .iter()
            .any(|p| Arc::ptr_eq(p, prop))
        {
            return true;
        }
        if let Some(last) = self.reverse_mapped_print_stack.last() {
            if let Some(links) = self.reverse_mapped_symbol_links.get(last) {
                if let Some(pt) = &links.property_type {
                    // 我们构建的命名接口带 symbol；匿名字面量无符号
                    if pt.symbol.is_some() {
                        return true;
                    }
                }
            }
        }
        if self.reverse_mapped_print_stack.len() < 3 {
            return false;
        }
        let Some(prop_links) = self.reverse_mapped_symbol_links.get(prop) else {
            return false;
        };
        let Some(prop_mapped_sym) = prop_links
            .mapped_type
            .as_ref()
            .and_then(|m| m.symbol.as_ref())
            .cloned()
        else {
            return false;
        };
        for i in 0..self.reverse_mapped_print_stack.len() {
            if i > 3 {
                break;
            }
            let entry = &self.reverse_mapped_print_stack[self.reverse_mapped_print_stack.len() - 1 - i];
            if let Some(links) = self.reverse_mapped_symbol_links.get(entry) {
                if links
                    .mapped_type
                    .as_ref()
                    .and_then(|m| m.symbol.as_ref())
                    .is_some_and(|s| Arc::ptr_eq(s, &prop_mapped_sym))
                {
                    return true;
                }
            }
        }
        false
    }
}
