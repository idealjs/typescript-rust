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

        {
            let own_mut = Arc::as_ptr(&own_type) as *mut crate::checker::types::Type;
            unsafe {
                if let TypeData::Object(obj) = &mut (*own_mut).data {
                    let sink = MemberSink {
                        members: &mut obj.structured.members,
                        props: &mut obj.structured.properties,
                    };
                    self.fill_members_into(sink, members);
                }
            }
        }

        let mut base_type: Option<Arc<Type>> = None;
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
        let result = match base_type {
            Some(base) => self.merge_instance_types(&own_type, &base),
            None => own_type,
        };
        self.class_instance_type_cache.insert(node_id, Arc::clone(&result));
        result
    }

    pub(crate) fn fill_members_into(&mut self, sink: MemberSink, members: &Arc<NodeList>) {
        for member in members.iter() {
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
                _ => {}
            }
        }
    }
}

pub(crate) struct MemberSink {
    pub(crate) members: *mut SymbolTable,
    pub(crate) props: *mut Vec<Arc<Symbol>>,
}
