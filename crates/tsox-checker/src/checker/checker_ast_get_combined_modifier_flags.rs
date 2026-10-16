#![allow(unused_imports)]

use crate::checker::checker::*;

pub(crate) fn ast_get_combined_modifier_flags(node: &Arc<Node>) -> ModifierFlags { ::tsox_core::fntrace::enter("ast_get_combined_modifier_flags"); 
    let current = Checker::get_root_declaration(node);
    let mut flags = current.syntactic_modifier_flags();
    if current.kind == SyntaxKind::VariableDeclaration {
        if let Some(parent) = current.parent() {
            if parent.kind == SyntaxKind::VariableDeclarationList {
                flags |= parent.syntactic_modifier_flags();
                if let Some(gp) = parent.parent() {
                    if gp.kind == SyntaxKind::VariableStatement {
                        flags |= gp.syntactic_modifier_flags();
                    }
                }
            }
        }
    }
    flags
}

impl Checker {
    pub fn mark_type_resolution_cycle(
        &mut self,
        target: *const Symbol,
        property: TypeResolutionProperty,
    ) { ::tsox_core::fntrace::enter("mark_type_resolution_cycle"); 
        if let Some(idx) = self
            .type_resolution_stack
            .iter()
            .rposition(|entry| entry.target == target && entry.property == property)
        {
            for entry in &mut self.type_resolution_stack[idx..] {
                entry.result = false;
            }
        }
    }

    pub fn push_type_resolution(
        &mut self,
        target: *const Symbol,
        property: TypeResolutionProperty,
    ) -> bool { ::tsox_core::fntrace::enter("push_type_resolution"); 
        // Go findResolutionCycleStartIndex（checker.go:19892-19904）只扫
        // resolutionStart 之上的窗口：getResolvedSignature（checker.go:8924）
        // 与 getVariancesWorker（relater.go:1433）把地板压到当前帧之下，
        // call 解析/方差计算对窗口外符号型的合法重入不构成环
        let floor = self.resolution_start.min(self.type_resolution_stack.len());
        let cycle_start = self
            .type_resolution_stack
            .get(floor..)
            .and_then(|window| {
                window
                    .iter()
                    .rposition(|entry| entry.target == target && entry.property == property)
            })
            .map(|offset| offset + floor);

        if let Some(idx) = cycle_start {
            // Go checkNodeDeferred/懒 resolvedReturnType：函数值定型期的体推断
            // 在 Go 中延后到外层符号帧出栈之后，帧下界以上的重入不构成环
            if self.rt_infer_boundary_marks.iter().any(|&m| m > idx) {
                return false;
            }
            self.mark_type_resolution_cycle(target, property);
            false
        } else {
            self.type_resolution_stack.push(TypeResolutionEntry {
                target,
                property,
                result: true,
            });
            true
        }
    }

    pub fn pop_type_resolution(&mut self) -> bool { ::tsox_core::fntrace::enter("pop_type_resolution"); 
        self.type_resolution_stack
            .pop()
            .map(|entry| entry.result)
            .unwrap_or(true)
    }

    pub fn is_resolving(&self, target: *const Symbol, property: TypeResolutionProperty) -> bool { ::tsox_core::fntrace::enter("is_resolving"); 
        self.type_resolution_stack
            .iter()
            .any(|entry| entry.target == target && entry.property == property)
    }
}
