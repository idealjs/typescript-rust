#![allow(unused_imports)]

use crate::checker::relater_relate_impl_chunk::*;
use tsox_frontend::ast::SyntaxKind;

impl Checker {
    /// Go isGenericMappedType（checker.go:25259）：约束为泛型索引类型，或 as
    /// 子句代入迭代参数后仍为泛型索引类型
    pub(crate) fn mapped_type_keys_are_generic(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("mapped_type_keys_are_generic"); 
        let TypeData::Mapped(m) = &t.data else {
            return false;
        };
        let constraint = m.constraint_type.clone();
        if constraint
            .as_ref()
            .is_some_and(|c| self.is_generic_index_type(c))
        {
            return true;
        }
        let (Some(name_type), Some(tp), Some(constraint)) =
            (m.name_type.clone(), m.type_parameter.clone(), constraint)
        else {
            return false;
        };
        let substituted =
            self.substitute_infer_type_parameters(&name_type, &[tp], &[constraint]);
        self.is_generic_index_type(&substituted)
    }

    /// Go isMappedTypeWithKeyofConstraintDeclaration（checker.go:23051）
    pub(crate) fn is_mapped_type_with_keyof_constraint_declaration(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_mapped_type_with_keyof_constraint_declaration");
        let TypeData::Mapped(m) = &t.data else {
            return false;
        };
        Self::mapped_constraint_is_bare_keyof(m)
    }

    /// Go isGenericMappedType（checker.go:26383-26395）：约束（经
    /// getConstraintTypeFromMappedType 含实例化壳 target+mapper 回退）为泛型
    /// 索引型，或 as 名型代入迭代参数后仍为泛型索引型。显示门用
    /// （createTypeNodeFromObjectType nodebuilderimpl.go:2869-2872）
    pub(crate) fn is_generic_mapped_type_for_display(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_generic_mapped_type_for_display");
        if let TypeData::Mapped(m) = &t.data {
            if m.contains_error.load(std::sync::atomic::Ordering::Relaxed) {
                return true;
            }
        } else {
            return false;
        }
        let Some(constraint) = self.get_constraint_type_from_mapped_type(t) else {
            return false;
        };
        if self.is_generic_index_type(&constraint) {
            return true;
        }
        let Some(name_type) = self.get_name_type_from_mapped_type(t) else {
            return false;
        };
        let Some(type_parameter) = self.get_type_parameter_from_mapped_type(t) else {
            return false;
        };
        let mapper = Arc::new(crate::checker::mapper::new_simple_type_mapper(
            type_parameter,
            Arc::clone(&constraint),
        ));
        let substituted = self.instantiate_type(&name_type, Some(&mapper));
        self.is_generic_index_type(&substituted)
    }

    /// Go getModifiersTypeFromMappedType（checker.go:28478）：实例化产物
    /// （mapper 非 None）时 keyof 操作数型 / 索引目标型按 mapper 实例化
    /// （checker.go:29796-29801/29808），泛型本体 mapper 为 None 恒等返回
    pub(crate) fn get_modifiers_type_from_mapped_type(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_modifiers_type_from_mapped_type");
        let TypeData::Mapped(m) = &t.data else {
            return self.unknown_type();
        };
        let chain = m.template_subst.as_ref().map(|c| c.as_ref().clone());
        let mapper = t.mapper().cloned();
        if Self::is_mapped_type_with_keyof_constraint_declaration(t) {
            if let Some(operand) = Self::mapped_keyof_operand_node(t) {
                let ty = self.get_type_from_type_node(&operand);
                // Go getModifiersTypeFromMappedType（checker.go:29801）：keyof 操作数
                // 按映射类型的 mapper 实例化——实例化壳由此从 keyof T 得到具体
                // modifiers 型（T→实参），模板替换链承担 ad-hoc 建型处的代换
                let ty = self.instantiate_type(&ty, t.mapper());
                return match chain {
                    Some(chain) => self.apply_template_subst_chain(&ty, &chain),
                    None => ty,
                };
                // Go getModifiersTypeFromMappedType（checker.go:29803）：
                // instantiateType(operand 型, m.mapper)——仅持 mapper 的实例化
                // 壳（无 template_subst 链）同需按 mapper 实参化，否则
                // modifiers 型滞留原始类型参数，成员 modifiers_prop 恒 miss
                //（optionality/声明链接全丢）
                return match mapper {
                    Some(mapper) => self.instantiate_type(&ty, Some(&mapper)),
                    None => ty,
                };
            }
            return self.unknown_type();
        }
        let constraint = self.get_constraint_type_from_mapped_type(t);
        let extended = match constraint {
            Some(c) if c.flags.contains(TypeFlags::TypeParameter) => {
                self.get_constraint_of_type_parameter(&c)
            }
            other => other,
        };
        if let Some(ext) = extended
            && let TypeData::Index(idx) = &ext.data
            && let Some(target) = &idx.target
        {
            let ty = Arc::clone(target);
            let ty = match &mapper {
                Some(mapper) => self.instantiate_type(&ty, Some(mapper)),
                None => ty,
            };
            return match chain {
                Some(chain) => self.apply_template_subst_chain(&ty, &chain),
                None => ty,
            };
            return match mapper {
                Some(mapper) => self.instantiate_type(&ty, Some(&mapper)),
                None => ty,
            };
        }
        self.unknown_type()
    }

    /// Go getApparentMappedTypeKeys（checker.go:23060）：modifiers 型的
    /// apparent 属性名/索引键逐个代入 nameType 的迭代参数后取并集
    pub(crate) fn get_apparent_mapped_type_keys(
        &mut self,
        name_type: &Arc<Type>,
        target_type: &Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_apparent_mapped_type_keys"); 
        let modifiers = self.get_modifiers_type_from_mapped_type(target_type);
        let apparent = self.get_apparent_type(&modifiers);
        let mut key_types: Vec<Arc<Type>> = self
            .get_properties_of_type(&apparent)
            .iter()
            .map(|prop| self.get_literal_type_from_property(prop))
            .collect();
        if apparent.flags.contains(TypeFlags::Any) {
            key_types.push(self.string_type());
        } else {
            for info in self.get_index_infos_of_type(&apparent) {
                if let Some(key) = &info.key_type {
                    key_types.push(Arc::clone(key));
                }
            }
        }
        let tp = self.get_type_parameter_from_mapped_type(target_type);
        let mapped_keys = key_types
            .into_iter()
            .map(|key| match &tp {
                Some(tp) => {
                    self.substitute_infer_type_parameters(name_type, &[Arc::clone(tp)], &[key])
                }
                None => Arc::clone(name_type),
            })
            .collect();
        self.get_union_type(mapped_keys)
    }

    /// Go relater.go:3740-3747（源侧）与 3553-3563（目标侧）共用的键集选择：
    /// keyof 声明约束取 apparent 代入键集，有 nameType 取 nameType，否则取约束
    pub(crate) fn mapped_deferred_key_set(
        &mut self,
        mapped: &Arc<Type>,
        include_name_type: bool,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("mapped_deferred_key_set"); 
        let name_type = self.get_name_type_from_mapped_type(mapped);
        let key_type = match &name_type {
            Some(nt) if Self::is_mapped_type_with_keyof_constraint_declaration(mapped) => {
                let mapped_keys = self.get_apparent_mapped_type_keys(nt, mapped);
                if include_name_type {
                    self.get_union_type(vec![mapped_keys, Arc::clone(nt)])
                } else {
                    mapped_keys
                }
            }
            Some(nt) => Arc::clone(nt),
            None => self.get_constraint_type_from_mapped_type(mapped)?,
        };
        Some(key_type)
    }

    pub(crate) fn mapped_keyof_operand_node(t: &Arc<Type>) -> Option<Arc<tsox_frontend::ast::Node>> { ::tsox_core::fntrace::enter("mapped_keyof_operand_node"); 
        let TypeData::Mapped(m) = &t.data else {
            return None;
        };
        let decl = m.declaration.as_ref()?;
        let tsox_frontend::ast::NodeData::MappedTypeNode(d) = &decl.data else {
            return None;
        };
        let constraint = match &d.type_parameter.data {
            tsox_frontend::ast::NodeData::TypeParameterDeclaration(td) => td.constraint.clone(),
            _ => None,
        }?;
        match &constraint.data {
            tsox_frontend::ast::NodeData::TypeOperatorNode(op)
                if op.operator == SyntaxKind::KeyOfKeyword =>
            {
                Some(Arc::clone(&op.type_node))
            }
            _ => None,
        }
    }
}
