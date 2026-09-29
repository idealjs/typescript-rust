#![allow(unused_imports)]

//! w9a: checker 余量收尾批(w9)
//!
//! 交接:printer::EF_NO_ASCII_ESCAPING 已接 m2g::r21k9_defs;
//! set_emit_flags 按 r28k8_defs::EmitContextStubExt28 存根化(EmitContextStub 无 emitNodes 侧表,对齐 r22k6 set_emit_flags_single_line 惯例);
//! clone_binding_name 中 visited 为 BindingElement 时 f.UpdateBindingElement 去掉初始化器一步待 NodeFactoryStub 补 update_binding_element;
//! trackComputedName 的 isLateBindableName 合取项待 Checker::is_late_bindable_name 转 pub + &self 后接管(当前仅按 IsComputedPropertyName 守卫,跟踪面偏宽,见 Go nodebuilderimpl.go cloneBindingName)。
//! 孪生裁决:
//! newWrappingTracker → nodecopy_recovery.rs WrappingTracker::new(已存在)
//! NewNodeBuilder/NewNodeBuilderEx → 架构差异:NodeBuilder 包装层未移植,new_node_builder_impl 已存在(m2f_3.rs)

#[path = "r28k8_defs.rs"]
pub mod r28k8_defs;

use tsox_frontend::ast::Node;
use crate::checker::mapper::{new_array_type_mapper, new_simple_type_mapper};
use crate::checker::nodecopy_builder::NodeBuilderImpl;
use crate::checker::symboltracker::NodeBuilderFlags;
use crate::checker::mig::m2f::r17k8_factory_ext::NodeFactoryExt;
use self::r28k8_defs::EmitContextStubExt28;
use crate::checker::mig::wc3::NodeAccessExt;
use std::sync::Arc;

pub(crate) fn new_type_mapper(sources: Vec<Arc<crate::checker::Type>>, targets: Vec<Arc<crate::checker::Type>>) -> crate::checker::TypeMapper {
    if sources.len() == 1 {
        let s = sources.into_iter().next().unwrap();
        let t = targets.into_iter().next().unwrap();
        new_simple_type_mapper(s, t)
    } else {
        new_array_type_mapper(sources, targets)
    }
}

impl<'a> NodeBuilderImpl<'a> {
    pub(crate) fn new_string_literal(&mut self, text: &str) -> Arc<Node> {
        self.new_string_literal_ex(text, false)
    }

    pub(crate) fn new_string_literal_ex(&mut self, text: &str, is_single_quote: bool) -> Arc<Node> {
        let mut flags: u32 = 0;
        if is_single_quote
            || self
                .ctx
                .borrow()
                .flags
                .intersects(NodeBuilderFlags::UseSingleQuotesForStringLiteralType)
        {
            flags |= tsox_frontend::scanner::TOKEN_FLAGS_SINGLE_QUOTE;
        }
        self.f.new_string_literal(text, flags as i32)
    }

    pub(crate) fn clone_binding_name(&mut self, node: &Arc<Node>) -> Arc<Node> {
        if tsox_frontend::ast::is_computed_property_name(node) {
            if let Some(expr) = node.expression() {
                let enclosing = self.ctx.borrow().enclosing_declaration.clone();
                self.track_computed_name(&expr, enclosing.as_ref());
            }
        }

        let visited = match self.clone_binding_name_visitor.as_ref() {
            Some(visitor) => (visitor.visit)(node).unwrap_or_else(|| Arc::clone(node)),
            None => Arc::clone(node),
        };

        if tsox_frontend::ast::node_data_generated::is_binding_element(&visited) {
            // Go: visited = b.f.UpdateBindingElement(..., nil /* remove initializer */)
            // NodeFactoryStub 尚无 update_binding_element,保持 visited 原样(见文件头交接)。
        }

        if !tsox_frontend::ast::node_is_synthesized(&visited) {
            self.deep_clone_node(&visited)
        } else {
            visited
        }
    }

    pub(crate) fn parameter_to_parameter_declaration_name(
        &mut self,
        parameter_symbol: &Arc<tsox_frontend::ast::Symbol>,
        parameter_declaration: Option<&Arc<Node>>,
    ) -> Option<Arc<Node>> {
        let parameter_declaration = parameter_declaration?;
        let name = parameter_declaration.name()?;
        match name.kind {
            tsox_frontend::ast::SyntaxKind::Identifier => {
                let cloned = self.deep_clone_node(&name);
                self.e.set_emit_flags(&cloned, crate::checker::mig::m2g::r21k9_defs::EF_NO_ASCII_ESCAPING);
                self.id_to_symbol.insert(cloned.id(), parameter_symbol.clone());
                Some(cloned)
            }
            tsox_frontend::ast::SyntaxKind::QualifiedName => {
                let right = name.as_qualified_name().right.clone();
                let cloned = self.deep_clone_node(&right);
                self.e.set_emit_flags(&cloned, crate::checker::mig::m2g::r21k9_defs::EF_NO_ASCII_ESCAPING);
                self.id_to_symbol.insert(cloned.id(), parameter_symbol.clone());
                Some(cloned)
            }
            _ => Some(self.clone_binding_name(&name)),
        }
    }
}
