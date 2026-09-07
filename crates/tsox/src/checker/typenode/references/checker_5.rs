#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn build_interface_type_from_members(
        &mut self,
        members: &Arc<NodeList>,
    ) -> Arc<Type> {
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        let mut index_infos: Vec<Arc<crate::checker::IndexInfo>> = Vec::new();

        let mut call_signatures: Vec<Arc<Signature>> = Vec::new();
        let mut construct_signatures: Vec<Arc<Signature>> = Vec::new();
        for member in members.iter() {
            match &member.data {
                NodeData::PropertySignatureDeclaration(_) => {
                    self.add_property_signature_member(member, &mut symbol_table, &mut props);
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
                    self.add_get_accessor_member(member, &mut symbol_table, &mut props);
                }
                NodeData::SetAccessorDeclaration(_) => {
                    self.add_set_accessor_member(member, &mut symbol_table, &mut props);
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

        let call_signature_count = call_signatures.len();
        let mut signatures = call_signatures;
        signatures.extend(construct_signatures);
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
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
