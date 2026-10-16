#![allow(unused_imports)]

use crate::checker::relater_probing::*;
use std::collections::HashMap;
use std::sync::Arc;
use tsox_frontend::ast::Node;

impl Checker {
    /// 条件型分支节点上下文化解析，对齐 Go getConditionalType 的分支求值形态
    /// instantiateType(getTypeFromTypeNode(TrueType), combinedMapper)
    /// （checker.go:25860/25882；getTrueTypeFromConditionalType
    /// checker.go:26020-26027 同形）。节点解析在单例世界恒无环境
    /// （typeNodeLinks.resolvedType 按节点驻留，getTypeFromTypeReference
    /// checker.go:24350-24361），语境由显式 mapper 在解析后施加。
    ///
    /// 环门控：首入（同分支节点未在解析栈中）按既有帧环境解析
    /// （type_argument_stack/name_frames 参与型参引用解析与
    /// (节点, stack_hash) subst 缓存分键，typenode_impl_chunk_checker.rs:32）；
    /// 同节点嵌套再入（自引用条件型环，__Awaited/Trim/_TupleOf 族）才清帧
    /// 窗解析——帧值环上每层不同（T→[1,2,3]→[2,3]→[3]），帧内解析键恒
    /// miss 无界重解析致 worker signal 6，清帧使 subst 缓存键退化为
    /// (节点,0)：环上第二次再入即命中，环自然有界；活帧语境按
    /// root.outer_type_parameters 域（Go 实例化 mapper 的映射域，
    /// getConditionalTypeInstantiation checker.go:23801-23803）捕获后经
    /// substitute_infer_type_parameters 显式施加，与别名体声明型解析后的
    /// 代入同机制（instantiate_alias_from_types_inner）。
    pub(crate) fn resolve_conditional_branch_node(
        &mut self,
        t: &Arc<Type>,
        node: &Arc<Node>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("resolve_conditional_branch_node");
        let node_key = node.id() as usize;
        if self.conditional_branch_nodes_resolving.insert(node_key) {
            let branch = self.get_type_from_type_node(node);
            self.conditional_branch_nodes_resolving.remove(&node_key);
            return branch;
        }
        let (outer_type_parameters, check_type_parameter_symbol) = match t
            .as_conditional_type()
            .and_then(|ct| ct.root.as_ref())
        {
            Some(root) => (
                root.outer_type_parameters.clone(),
                root.check_type_parameter_symbol.clone(),
            ),
            None => (Vec::new(), None),
        };
        let mut merged: HashMap<usize, Arc<Type>> = HashMap::new();
        for frame in &self.type_argument_name_frames {
            for (sym, v) in frame {
                merged.insert(Arc::as_ptr(sym) as usize, Arc::clone(v));
            }
        }
        for frame in &self.type_argument_stack {
            for (k, v) in frame {
                merged.insert(*k as usize, Arc::clone(v));
            }
        }
        let mut candidate_tps = outer_type_parameters;
        if let Some(sym) = &check_type_parameter_symbol
            && !candidate_tps
                .iter()
                .any(|p| p.symbol.as_ref().is_some_and(|s| Arc::ptr_eq(s, sym)))
        {
            candidate_tps.push(self.get_type_parameter_from_symbol(sym));
        }
        let mut tps: Vec<Arc<Type>> = Vec::new();
        let mut args: Vec<Arc<Type>> = Vec::new();
        for tp in &candidate_tps {
            let Some(sym) = tp.symbol.as_ref() else {
                continue;
            };
            if tps
                .iter()
                .any(|p| p.symbol.as_ref().is_some_and(|s| Arc::ptr_eq(s, sym)))
            {
                continue;
            }
            let Some(mut v) = merged
                .get(&(Arc::as_ptr(sym) as usize))
                .cloned()
                .or_else(|| {
                    // 符号实例漂移回退：帧键与捕获符号不同实例时按名字+容器
                    // 等价命中（resolve_type_parameter_reference 同款判定）
                    merged
                        .iter()
                        .find(|(k, _)| {
                            let frame_sym =
                                unsafe { &*(**k as *const tsox_frontend::ast::Symbol) };
                            self.type_param_symbols_equivalent(frame_sym, sym)
                        })
                        .map(|(_, v)| Arc::clone(v))
                })
            else {
                continue;
            };
            // 环境值链式跟随（resolve_type_parameter_reference 同款）：帧值为
            // 更外层帧绑定的类型参数时逐层追到底，等价单趟多参数代入
            for _ in 0..=merged.len() {
                if !v.is_type_parameter() {
                    break;
                }
                let Some(vs) = v.symbol.as_ref() else {
                    break;
                };
                let Some(next) = merged.get(&(Arc::as_ptr(vs) as usize)) else {
                    break;
                };
                if Arc::ptr_eq(next, &v) {
                    break;
                }
                v = Arc::clone(next);
            }
            tps.push(Arc::clone(tp));
            args.push(v);
        }
        if tps.is_empty()
            && self.type_argument_stack.is_empty()
            && self.type_argument_name_frames.is_empty()
        {
            return self.get_type_from_type_node(node);
        }
        let saved_stack = std::mem::take(&mut self.type_argument_stack);
        let saved_frames = std::mem::take(&mut self.type_argument_name_frames);
        let declared = self.get_type_from_type_node(node);
        self.type_argument_stack = saved_stack;
        self.type_argument_name_frames = saved_frames;
        if tps.is_empty() || !self.could_contain_type_variables(&declared) {
            return declared;
        }
        self.substitute_infer_type_parameters(&declared, &tps, &args)
    }
}
