#![allow(unused_imports)]

use crate::checker::typenode_references::*;

impl Checker {
    pub(crate) fn build_interface_type_from_members(
        &mut self,
        members: &Arc<NodeList>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("build_interface_type_from_members"); 
        self.build_interface_type_from_members_with_symbol(members, None, None)
    }

    pub(crate) fn build_interface_type_from_members_with_symbol(
        &mut self,
        members: &Arc<NodeList>,
        symbol: Option<Arc<Symbol>>,
        own_symbol: Option<&Arc<Symbol>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("build_interface_type_from_members_with_symbol"); 
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        let mut index_infos: Vec<Arc<crate::checker::IndexInfo>> = Vec::new();

        let mut call_signatures: Vec<Arc<Signature>> = Vec::new();
        let mut construct_signatures: Vec<Arc<Signature>> = Vec::new();
        let mut implied_index: Option<crate::checker::IndexInfo> = None;
        let mut deferred_accessors: Vec<Arc<Node>> = Vec::new();
        for member in members.iter() {
            if implied_index.is_none()
                && matches!(
                    &member.data,
                    NodeData::PropertySignatureDeclaration(_)
                        | NodeData::MethodSignatureDeclaration(_)
                        | NodeData::PropertyDeclaration(_)
                        | NodeData::MethodDeclaration(_)
                        | NodeData::GetAccessorDeclaration(_)
                        | NodeData::SetAccessorDeclaration(_)
                )
            {
                implied_index = self.implied_index_info_of_computed_member(member, &index_infos);
            }
            match &member.data {
                NodeData::PropertySignatureDeclaration(_) => {
                    self.add_property_signature_member(
                        member,
                        &mut symbol_table,
                        &mut props,
                        own_symbol,
                    );
                }
                NodeData::MethodSignatureDeclaration(_) => {
                    self.add_method_signature_member(member, &mut symbol_table, &mut props);
                }
                NodeData::IndexSignatureDeclaration(_) => {
                    self.add_index_signature_member(member, &mut index_infos);
                }
                NodeData::PropertyDeclaration(_) => {
                    self.add_property_declaration_member(member, &mut symbol_table, &mut props);
                }
                NodeData::MethodDeclaration(_) => {
                    self.add_method_declaration_member(member, &mut symbol_table, &mut props);
                }
                NodeData::GetAccessorDeclaration(_) => {
                    self.add_get_accessor_member(member, &mut symbol_table, &mut props, &mut deferred_accessors);
                }
                NodeData::SetAccessorDeclaration(_) => {
                    self.add_set_accessor_member(member, &mut symbol_table, &mut props, &mut deferred_accessors);
                }
                NodeData::CallSignatureDeclaration(_) => {
                    self.add_call_signature_member(member, &mut call_signatures);
                }
                NodeData::ConstructSignatureDeclaration(_) => {
                    self.add_construct_signature_member(member, &mut construct_signatures);
                }
                NodeData::ConstructorDeclaration(_) => {
                    self.add_constructor_properties(member, &mut symbol_table, &mut props);
                }
                _ => {}
            }
        }
        if let Some(info) = implied_index
            && !index_infos.iter().any(|i| {
                i.key_type
                    .as_ref()
                    .is_some_and(|k| Some(k.flags) == info.key_type.as_ref().map(|kk| kk.flags))
            })
        {
            index_infos.push(Arc::new(info));
        }
        // 接口声明型构造不急切解析访问器成员型（Go resolveStructuredTypeMembers
        // 惰性、getTypeOfAccessors 按需带帧）：接口构建帧内预解析会在成员注解
        // 求值中回指宿主容器（get bestFriend(): User → resolve_interface_type_ex
        // 重入本 DeclaredType 帧）计环，成员取型 pop 失败误报 TS2502 并驻留 any
        // （mappedTypeCircularReferenceInAccessor）——接口成员留待
        // get_type_of_symbol 按需定型；类/类型字面量容器维持预解析
        if !deferred_accessors.is_empty()
            && !deferred_accessors
                .first()
                .and_then(|m| m.parent())
                .as_deref()
                .is_some_and(|p| p.kind == SyntaxKind::InterfaceDeclaration)
        {
            self.resolve_deferred_accessor_member_types(&deferred_accessors, &symbol_table);
        }

        let call_signature_count = call_signatures.len();
        let mut signatures = call_signatures;
        signatures.extend(construct_signatures);
                Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous | ObjectFlags::Interface,
            id: crate::checker::types::next_type_id(),
            symbol,
            alias: None,
            data: TypeData::Object(ObjectTypeData { node: None,
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    index_infos,
                    signatures,
                    call_signature_count,
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }
}
