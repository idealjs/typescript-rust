#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn type_to_type_node_worker(&mut self, t: &Arc<Type>) -> Arc<Node> {
        if let Some(name) = t.intrinsic_name() {
            return self.intrinsic_to_type_node(name);
        }

        if let Some(val) = t.literal_value() {
            return self.literal_value_to_type_node(val);
        }

        if t.flags.contains(TypeFlags::UniqueESSymbol) {
            return self.type_operator_node(
                SyntaxKind::UniqueKeyword,
                self.keyword_node(SyntaxKind::SymbolKeyword),
            );
        }

        if t.flags.contains(TypeFlags::Never) {
            return self.keyword_node(SyntaxKind::NeverKeyword);
        }

        if t.is_union() {
            return self.union_to_type_node(t);
        }

        if t.is_intersection() {
            return self.intersection_to_type_node(t);
        }

        if t.is_type_parameter() {
            return self.type_parameter_to_type_node(t);
        }

        if t.object_flags.contains(ObjectFlags::Tuple) {
            return self.tuple_to_type_node(t);
        }

        if t.object_flags.contains(ObjectFlags::Reference) {
            return self.reference_to_type_node(t);
        }

        if let Some(structured) = t.as_structured() {
            if structured.call_signature_count > 0 && t.symbol.is_none() {
                return self.function_type_to_type_node(structured);
            }
        }

        if let Some(sym) = &t.symbol {
            let instance_args = t.as_object().and_then(|obj| {
                (!obj.type_arguments.is_empty()).then(|| {
                    let arg_nodes: Vec<Arc<Node>> = obj
                        .type_arguments
                        .iter()
                        .map(|ty| self.type_to_type_node(ty))
                        .collect();
                    Arc::new(NodeList::new(arg_nodes))
                })
            });
            return self.symbol_to_type_node(sym, SymbolFlags::TYPE, instance_args);
        }

        if let Some(structured) = t.as_structured() {
            if !structured.properties.is_empty()
                || !structured.call_signatures().is_empty()
                || !structured.index_infos.is_empty()
            {
                return self.type_literal_to_type_node(structured);
            }
        }

        if t.flags.contains(TypeFlags::Object) {
            return self.keyword_node(SyntaxKind::ObjectKeyword);
        }
        if t.flags.contains(TypeFlags::Unknown) {
            return self.keyword_node(SyntaxKind::UnknownKeyword);
        }

        self.keyword_node(SyntaxKind::AnyKeyword)
    }

    pub(crate) fn intrinsic_to_type_node(&mut self, name: &str) -> Arc<Node> {
        let kind = match name {
            "any" => SyntaxKind::AnyKeyword,
            "unknown" => SyntaxKind::UnknownKeyword,
            "string" => SyntaxKind::StringKeyword,
            "number" => SyntaxKind::NumberKeyword,
            "bigint" => SyntaxKind::BigIntKeyword,
            "boolean" => SyntaxKind::BooleanKeyword,
            "symbol" => SyntaxKind::SymbolKeyword,
            "void" => SyntaxKind::VoidKeyword,
            "undefined" => SyntaxKind::UndefinedKeyword,
            "null" => SyntaxKind::NullKeyword,
            "object" => SyntaxKind::ObjectKeyword,
            "never" => SyntaxKind::NeverKeyword,

            _ => SyntaxKind::AnyKeyword,
        };
        self.keyword_node(kind)
    }

    pub(crate) fn literal_value_to_type_node(&mut self, val: &LiteralValue) -> Arc<Node> {
        let literal = match val {
            LiteralValue::String(s) => self.string_literal_node(s),
            LiteralValue::Number(n) => self.numeric_literal_node(&n.to_string()),
            LiteralValue::BigInt(b) => self.bigint_literal_node(&b.to_string()),
            LiteralValue::Boolean(true) => self.keyword_node(SyntaxKind::TrueKeyword),
            LiteralValue::Boolean(false) => self.keyword_node(SyntaxKind::FalseKeyword),

            LiteralValue::None => return self.keyword_node(SyntaxKind::NullKeyword),
        };
        self.literal_type_node(literal)
    }

    pub(crate) fn union_to_type_node(&mut self, t: &Arc<Type>) -> Arc<Node> {
        let types = t.types().unwrap_or(&[]);
        if types.is_empty() {
            return self.keyword_node(SyntaxKind::NeverKeyword);
        }
        if types.len() == 1 {
            return self.type_to_type_node(&types[0]);
        }

        let mut ordered: Vec<&Arc<Type>> = Vec::with_capacity(types.len());
        let mut nulls: Vec<&Arc<Type>> = Vec::new();
        let mut undefs: Vec<&Arc<Type>> = Vec::new();
        for ty in types.iter() {
            if ty.flags.contains(TypeFlags::Undefined) {
                undefs.push(ty);
            } else if ty.flags.contains(TypeFlags::Null) {
                nulls.push(ty);
            } else {
                ordered.push(ty);
            }
        }
        ordered.extend(nulls);
        ordered.extend(undefs);
        let nodes: Vec<Arc<Node>> = ordered
            .into_iter()
            .map(|ty| {
                let node = self.type_to_type_node(ty);
                if self.needs_parens_in_union(ty) {
                    self.parenthesized_type_node(node)
                } else {
                    node
                }
            })
            .collect();
        self.union_type_node(nodes)
    }

    pub(crate) fn intersection_to_type_node(&mut self, t: &Arc<Type>) -> Arc<Node> {
        let types = t.types().unwrap_or(&[]);
        if types.is_empty() {
            return self.keyword_node(SyntaxKind::UnknownKeyword);
        }
        if types.len() == 1 {
            return self.type_to_type_node(&types[0]);
        }
        let nodes: Vec<Arc<Node>> = types
            .iter()
            .map(|ty| {
                let node = self.type_to_type_node(ty);
                if self.needs_parens_in_union(ty) {
                    self.parenthesized_type_node(node)
                } else {
                    node
                }
            })
            .collect();
        self.intersection_type_node(nodes)
    }

    pub(crate) fn type_parameter_to_type_node(&mut self, t: &Arc<Type>) -> Arc<Node> {
        if let TypeData::TypeParameter(tp) = &t.data {
            if tp.is_this_type {
                let name = self.identifier("this");
                return self.type_reference_node(name, None);
            }
        }
        if let Some(sym) = &t.symbol {
            let name = self.identifier(&sym.name);
            return self.type_reference_node(name, None);
        }
        let name = self.identifier("T");
        self.type_reference_node(name, None)
    }

    pub(crate) fn tuple_to_type_node(&mut self, t: &Arc<Type>) -> Arc<Node> {
        let TypeData::Tuple(tuple) = &t.data else {
            return self.tuple_type_node(Vec::new());
        };
        let elements: Vec<Arc<Node>> = tuple
            .element_infos
            .iter()
            .map(|elem| {
                let ty = elem
                    .type_
                    .as_ref()
                    .map(|ty| self.type_to_type_node(ty))
                    .unwrap_or_else(|| self.keyword_node(SyntaxKind::AnyKeyword));

                if elem.flags.contains(ElementFlags::Rest)
                    || elem.flags.contains(ElementFlags::Variadic)
                {
                    self.rest_type_node(ty)
                } else {
                    ty
                }
            })
            .collect();
        self.tuple_type_node(elements)
    }

    pub(crate) fn reference_to_type_node(&mut self, t: &Arc<Type>) -> Arc<Node> {
        let obj_data = match &t.data {
            TypeData::Object(o) => o,
            TypeData::Interface(i) => &i.object,
            _ => return self.keyword_node(SyntaxKind::ObjectKeyword),
        };

        let symbol_name = t.symbol.as_ref().map(|s| s.name.as_str()).unwrap_or("");
        let is_array = obj_data.type_arguments.len() == 1
            && (symbol_name == "Array" || symbol_name == "ReadonlyArray" || t.symbol.is_none());

        if is_array {
            let elem = &obj_data.type_arguments[0];
            let elem_node = self.type_to_type_node(elem);
            if self.needs_parens_in_union(elem) {
                return self.array_type_node(self.parenthesized_type_node(elem_node));
            }

            return self.array_type_node(elem_node);
        }

        let name = if symbol_name.is_empty() {
            self.identifier("object")
        } else {
            self.identifier(symbol_name)
        };
        let type_args = if obj_data.type_arguments.is_empty() {
            None
        } else {
            let arg_nodes: Vec<Arc<Node>> = obj_data
                .type_arguments
                .iter()
                .map(|ty| self.type_to_type_node(ty))
                .collect();
            Some(Arc::new(NodeList::new(arg_nodes)))
        };
        self.type_reference_node(name, type_args)
    }

    pub(crate) fn function_type_to_type_node(&mut self, structured: &StructuredTypeData) -> Arc<Node> {
        let sigs = structured.call_signatures();
        if sigs.is_empty() {
            let ret = self.keyword_node(SyntaxKind::UnknownKeyword);
            return self.function_type_node(Vec::new(), ret);
        }
        let sig = &sigs[0];
        let params = self.signature_to_parameter_nodes(sig);
        let ret_type = sig
            .resolved_return_type
            .get()
            .cloned()
            .unwrap_or_else(|| self.any_type());
        let ret_node = self.type_to_type_node(&ret_type);
        self.function_type_node(params, ret_node)
    }

    pub(crate) fn type_literal_to_type_node(&mut self, structured: &StructuredTypeData) -> Arc<Node> {
        let mut members: Vec<Arc<Node>> = Vec::new();

        for sig in structured.call_signatures() {
            members.push(self.call_signature_to_node(sig));
        }

        for prop in &structured.properties {
            let name = self.identifier(&prop.name);
            let prop_type = self.get_type_of_symbol(prop);
            let type_node = self.type_to_type_node(&prop_type);
            let optional = prop.flags.contains(SymbolFlags::Optional);
            members.push(self.property_signature_node(name, optional, type_node));
        }

        self.type_literal_node(members)
    }

}
