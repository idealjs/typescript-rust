#![allow(unused_imports)]

use crate::checker::checker_imports_namespace::*;

impl Checker {
    pub fn build_class_instance_type_with_base(&mut self, node: &Arc<Node>) -> Arc<Type> {
        // Go getDeclaredTypeOfClassOrInterface：类型先驻留后解析成员；成员解析中
        // 重入的 this 表达式拿到在造类型（已解析的成员可见，Go UnresolvedMembers）
        let node_id = node.id();
        if let Some(t) = self.class_instance_type_cache.get(&node_id) {
            return Arc::clone(t);
        }

        let (members, heritage_clauses) = match &node.data {
            tsox_frontend::ast::NodeData::ClassDeclaration(data) => {
                (&data.members, data.heritage_clauses.clone())
            }

            tsox_frontend::ast::NodeData::ClassExpression(data) => {
                (&data.members, data.heritage_clauses.clone())
            }
            _ => return self.build_interface_type_from_members(&Arc::new(NodeList::default())),
        };

        let own_type = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),
            symbol: self.program.symbol_map().symbol_of(node).cloned(),
            alias: None,
            data: TypeData::Object(ObjectTypeData::default()),
        });
        self.class_instance_type_cache
            .insert(node_id, Arc::clone(&own_type));
        self.class_build_in_progress.insert(node_id);

        {
            let own_mut = Arc::as_ptr(&own_type) as *mut crate::checker::types::Type;
            self.filling_class_members.insert(node_id);
            unsafe {
                if let TypeData::Object(obj) = &mut (*own_mut).data {
                    let sink = MemberSink {
                        members: &mut obj.structured.members,
                        props: &mut obj.structured.properties,
                        index_infos: &mut obj.structured.index_infos,
                    };
                    self.fill_members_into(sink, members);
                }
            }
            self.filling_class_members.remove(&node_id);

            // 同名 interface 声明（同文件或模块增强合并进来）的成员并入实例型
            // （Go getDeclaredTypeOfClassOrInterface 的 interface declaration 分支）
            let extra_interface_members: Vec<Arc<Node>> = self
                .program
                .symbol_map()
                .symbol_of(node)
                .map(|sym| {
                    sym.declarations
                        .iter()
                        .filter(|d| d.kind == SyntaxKind::InterfaceDeclaration && !Arc::ptr_eq(d, node))
                        .filter_map(|d| match &d.data {
                            NodeData::InterfaceDeclaration(id) => {
                                Some(id.members.iter().cloned().collect::<Vec<_>>())
                            }
                            _ => None,
                        })
                        .flatten()
                        .collect()
                })
                .unwrap_or_default();
            if !extra_interface_members.is_empty() {
                let mut nodes = Vec::new();
                nodes.extend(extra_interface_members);
                let list = Arc::new(NodeList::new(nodes));
                self.filling_class_members.insert(node_id);
                unsafe {
                    if let TypeData::Object(obj) = &mut (*own_mut).data {
                        let sink = MemberSink {
                            members: &mut obj.structured.members,
                            props: &mut obj.structured.properties,
                            index_infos: &mut obj.structured.index_infos,
                        };
                        self.fill_members_into(sink, &list);
                    }
                }
                self.filling_class_members.remove(&node_id);
            }
        }

        let mut base_type: Option<Arc<Type>> = None;
        let has_extends = heritage_clauses.as_ref().is_some_and(|h| {
            h.iter().any(|clause| {
                matches!(
                    &clause.data,
                    tsox_frontend::ast::NodeData::HeritageClause(hc)
                        if hc.token == SyntaxKind::ExtendsKeyword
                )
            })
        });
        let base_guard_pushed = has_extends
            && own_type.symbol.as_ref().is_some_and(|sym| {
                self.push_type_resolution(
                    Arc::as_ptr(sym) as *const tsox_frontend::ast::Symbol,
                    TypeResolutionProperty::ResolvedBaseTypes,
                )
            });
        if let Some(ref heritage) = heritage_clauses {
            for clause in heritage.iter() {
                if let tsox_frontend::ast::NodeData::HeritageClause(hc) = &clause.data {
                    if hc.token == SyntaxKind::ExtendsKeyword {
                        if let Some(type_ref) = hc.types.iter().next() {
                            base_type = Some(self.resolve_base_class_instance_type(type_ref));
                        }
                        break;
                    }
                }
            }
        }
        if base_guard_pushed && !self.pop_type_resolution() {
            if let Some(sym) = own_type.symbol.clone() {
                self.emit_ts2506(node, &sym);
            }
        }
        if let Some(base) = base_type {
            let base_still_building = base
                .symbol
                .as_ref()
                .and_then(|s| {
                    s.declarations
                        .iter()
                        .find(|d| matches!(d.data, NodeData::ClassDeclaration(_)))
                })
                .is_some_and(|d| self.class_build_in_progress.contains(&d.id()));
            if base_still_building {
                self.pending_base_merges.push((node_id, base));
            } else {
                self.merge_base_into_shell(&own_type, &base);
            }
        }
        self.class_build_in_progress.remove(&node_id);
        self.drain_pending_base_merges();
        self.class_instance_type_cache.insert(node_id, Arc::clone(&own_type));
        own_type
    }

    fn merge_base_into_shell(&mut self, own_type: &Arc<Type>, base: &Arc<Type>) {
        // 就地合并：成员填期的早前引用（自引用返回型等）持有的是壳本身，
        // 合并结果必须写回同一 Arc，否则早前引用永远看不到基类成员
        let merged = self.merge_instance_types(own_type, base);
        if !Arc::ptr_eq(&merged, own_type)
            && let Some(merged_struct) = merged.as_structured()
        {
            unsafe {
                let ptr = Arc::as_ptr(own_type) as *mut crate::checker::types::Type;
                if let crate::checker::types::TypeData::Object(obj) = &mut (*ptr).data {
                    obj.structured.members = merged_struct.members.clone();
                    obj.structured.properties = merged_struct.properties.clone();
                    obj.structured.index_infos = merged_struct.index_infos.clone();
                    obj.structured.signatures = merged_struct.signatures.clone();
                    obj.structured.call_signature_count = merged_struct.call_signature_count;
                }
            }
        }
    }

    fn base_merge_ready(&self, base: &Arc<Type>) -> bool {
        let Some(sym) = base.symbol.as_ref() else {
            return true;
        };
        let Some(decl) = sym
            .declarations
            .iter()
            .find(|d| matches!(d.data, NodeData::ClassDeclaration(_)))
        else {
            return true;
        };
        let id = decl.id();
        !self.class_build_in_progress.contains(&id)
            && !self.pending_base_merges.iter().any(|(d, _)| *d == id)
    }

    fn drain_pending_base_merges(&mut self) {
        loop {
            let Some(idx) = self
                .pending_base_merges
                .iter()
                .position(|(_, base)| self.base_merge_ready(base))
            else {
                break;
            };
            let (derived_id, base) = self.pending_base_merges.remove(idx);
            let Some(derived) = self.class_instance_type_cache.get(&derived_id).cloned() else {
                continue;
            };
            self.merge_base_into_shell(&derived, &base);
        }
    }

    pub(crate) fn single_base_for_non_augmenting_subtype(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let mut cur = Arc::clone(t);
        for _ in 0..32 {
            let Some(sym) = cur.symbol.clone() else {
                break;
            };
            let Some(node) = sym
                .declarations
                .iter()
                .find(|d| {
                    matches!(
                        d.data,
                        NodeData::ClassDeclaration(_) | NodeData::ClassExpression(_)
                    )
                })
                .cloned()
            else {
                break;
            };
            let (members, heritage) = match &node.data {
                NodeData::ClassDeclaration(d) => (&d.members, &d.heritage_clauses),
                NodeData::ClassExpression(d) => (&d.members, &d.heritage_clauses),
                _ => break,
            };
            if !members.is_empty() {
                break;
            }
            let Some(heritage) = heritage else {
                break;
            };
            let mut base = None;
            for clause in heritage.iter() {
                if let NodeData::HeritageClause(hc) = &clause.data
                    && hc.token == SyntaxKind::ExtendsKeyword
                {
                    base = hc.types.iter().next().cloned();
                    break;
                }
            }
            let Some(type_ref) = base else {
                break;
            };
            let bt = self.resolve_base_class_instance_type(&type_ref);
            if Arc::ptr_eq(&bt, &cur) || bt.symbol.as_ref().is_none_or(|s| Arc::ptr_eq(s, &sym)) {
                break;
            }
            cur = bt;
        }
        cur
    }

    pub(crate) fn fill_members_into(&mut self, sink: MemberSink, members: &Arc<NodeList>) {
        let mut implied_index: Option<crate::checker::IndexInfo> = None;
        for member in members.iter() {
            if implied_index.is_none() {
                implied_index = self.implied_index_info_of_computed_member(member, unsafe {
                    &*sink.index_infos
                });
            }
            let (tbl, props) = unsafe { (&mut *sink.members, &mut *sink.props) };
            match &member.data {
                NodeData::PropertySignatureDeclaration(_) => {
                    self.add_property_signature_member(member, tbl, props);
                }
                NodeData::MethodSignatureDeclaration(_) => {
                    self.add_method_signature_member(member, tbl, props);
                }
                NodeData::PropertyDeclaration(_) => {
                    self.add_property_declaration_member(member, tbl, props);
                }
                NodeData::MethodDeclaration(_) => {
                    self.add_method_declaration_member(member, tbl, props);
                }
                NodeData::GetAccessorDeclaration(_) => {
                    self.add_get_accessor_member(member, tbl, props);
                }
                NodeData::SetAccessorDeclaration(_) => {
                    self.add_set_accessor_member(member, tbl, props);
                }
                NodeData::ConstructorDeclaration(_) => {
                    self.add_constructor_properties(member, tbl, props);
                }
                NodeData::IndexSignatureDeclaration(_) => {
                    if !member.has_syntactic_modifier(ModifierFlags::Static) {
                        let infos = unsafe { &mut *sink.index_infos };
                        self.add_index_signature_member(member, infos);
                    }
                }
                _ => {}
            }
        }
        if let Some(info) = implied_index {
            let infos = unsafe { &mut *sink.index_infos };
            infos.push(Arc::new(info));
        }
    }
}

pub(crate) struct MemberSink {
    pub(crate) members: *mut SymbolTable,
    pub(crate) props: *mut Vec<Arc<Symbol>>,
    pub(crate) index_infos: *mut Vec<Arc<crate::checker::IndexInfo>>,
}
