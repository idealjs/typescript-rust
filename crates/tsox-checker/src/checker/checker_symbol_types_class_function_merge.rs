#![allow(unused_imports)]

use crate::checker::checker_symbol_types::*;

impl Checker {
    pub(crate) fn merged_class_function_symbol_type(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Type>> {
        let class_decl = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ClassDeclaration)?;
        let fn_decl = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::FunctionDeclaration)?;

        let ctor_type = self.get_type_of_class_declaration(class_decl);
        let fn_type = self
            .build_overload_function_type(symbol)
            .unwrap_or_else(|| self.get_type_of_function_like(fn_decl));

        let ctor_obj = ctor_type.as_structured()?;
        let fn_obj = fn_type.as_structured()?;
        let call_sigs: Vec<Arc<Signature>> = fn_obj.call_signatures().to_vec();

        let ctor_structured = ctor_obj;
        let mut structured = StructuredTypeData::default();
        structured.members = ctor_structured.members.clone();
        structured.properties = ctor_structured.properties.clone();
        structured.index_infos = ctor_structured.index_infos.clone();

        let existing_sigs = ctor_structured.signatures.clone();
        let existing_call_count = ctor_structured.call_signature_count;
        structured.call_signature_count = call_sigs.len() + existing_call_count;
        structured.signatures = call_sigs;
        structured
            .signatures
            .extend(existing_sigs[..existing_call_count].to_vec());
        structured
            .signatures
            .extend(existing_sigs[existing_call_count..].to_vec());

        let merged = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured,
                target: None,
                mapper: None,
                type_arguments: Vec::new(),
            }),
        });
        Some(self.attach_function_expando_type(symbol, merged))
    }
}
