#![allow(unused_imports)]
use crate::checker::mig::m1f::r19k9_defs::R19K9NodeExt;

#[allow(unused_imports, ambiguous_glob_reexports)]
use crate::checker::*;
#[allow(unused_imports)]
use tsox_frontend::ast::*;
#[allow(unused_imports)]
use tsox_core::diagnostics::messages_generated::*;
use tsox_core::diagnostics::Message;

pub(crate) use crate::checker::checker::*;
pub(crate) use super::m1f::*;
#[allow(unused_imports)]
use crate::checker::mig::m1g_5::get_type_list_key;
#[allow(unused_imports)]
use crate::checker::mig::m2a::is_spread_argument;
#[allow(unused_imports)]
use crate::checker::mig::m1b::function_like_data_full_signature;
use crate::checker::mig::m1f::r25k6_defs::map_type_with_checker;
#[allow(unused_imports)]
use crate::checker::exports_union_reduction::get_declaration_modifier_flags_from_symbol;
#[allow(unused_imports)]
use tsox_frontend::ast::mig::m3e_4::get_declaration_of_kind;
#[allow(unused_imports)]
use tsox_core::core::core::find_index;
use std::sync::Arc;

pub(crate) struct IterationTypesResolver {
    pub iterator_symbol_name: String,
    pub get_global_iterator_type: Box<dyn Fn() -> Option<Arc<Type>>>,
    pub get_global_iterable_type: Box<dyn Fn() -> Option<Arc<Type>>>,
    pub get_global_iterable_type_checked: Box<dyn Fn() -> Option<Arc<Type>>>,
    pub get_global_iterable_iterator_type: Box<dyn Fn() -> Option<Arc<Type>>>,
    pub get_global_iterable_iterator_type_checked: Box<dyn Fn() -> Option<Arc<Type>>>,
    pub get_global_iterator_object_type: Box<dyn Fn() -> Option<Arc<Type>>>,
    pub get_global_generator_type: Box<dyn Fn() -> Option<Arc<Type>>>,
    pub get_global_builtin_iterator_types: Box<dyn Fn() -> Vec<Arc<Type>>>,
    pub resolve_iteration_type: Box<dyn Fn(&Arc<Type>, Option<&Node>) -> Option<Arc<Type>>>,
    pub must_have_a_next_method_diagnostic: Option<&'static Message>,
    pub must_be_a_method_diagnostic: Option<&'static Message>,
}

pub(crate) struct IterationTypes {
    pub yield_type: Option<Arc<Type>>,
    pub return_type: Option<Arc<Type>>,
    pub next_type: Option<Arc<Type>>,
}

impl IterationTypesResolver {
    pub fn get_resolved_iteration_types(
        &self,
        yield_type: &Arc<Type>,
        return_type: &Arc<Type>,
        next_type: Option<Arc<Type>>,
    ) -> IterationTypes {
        IterationTypes {
            yield_type: Some(
                (self.resolve_iteration_type)(yield_type, None).unwrap_or_else(|| yield_type.clone()),
            ),
            return_type: Some(
                (self.resolve_iteration_type)(return_type, None)
                    .unwrap_or_else(|| return_type.clone()),
            ),
            next_type,
        }
    }
}

impl Checker {
    pub fn get_rest_type(
        &mut self,
        source: &Arc<Type>,
        properties: &[Arc<Node>],
        symbol: Option<&Arc<Symbol>>,
    ) -> Arc<Type> {
        let source = self.filter_type(source, &mut |t: &Arc<Type>| {
            !t.flags.intersects(TypeFlags::NULLABLE)
        });
        if source.flags.intersects(TypeFlags::Never) {
            return self.empty_object_type();
        }
        if source.flags.intersects(TypeFlags::Union) {
            return map_type_with_checker(self, &source, &mut |c, t| {
                Some(c.get_rest_type(t, properties, symbol))
            })
            .unwrap_or_else(|| source.clone());
        }
        let mut omit_key_types: Vec<Arc<Type>> = Vec::new();
        for p in properties {
            if let Some(t) = self.get_literal_type_from_property_name(p) {
                omit_key_types.push(t);
            }
        }
        let mut omit_key_type = self.get_union_type(omit_key_types);
        let mut spreadable_properties: Vec<Arc<Symbol>> = vec![];
        let mut unspreadable_to_rest_keys: Vec<Arc<Type>> = vec![];
        for prop in self.get_properties_of_type(&source) {
            let literal_type_from_property = self.get_literal_type_from_property(&prop);
            if !self.is_type_assignable_to(&literal_type_from_property, &omit_key_type)
                && !get_declaration_modifier_flags_from_symbol(&prop)
                    .intersects(ModifierFlags::Private | ModifierFlags::Protected)
                && self.is_spreadable_property(&prop)
            {
                spreadable_properties.push(prop);
            } else {
                unspreadable_to_rest_keys.push(literal_type_from_property);
            }
        }
        if self.is_generic_object_type(&source) || self.is_generic_index_type(&omit_key_type) {
            if !unspreadable_to_rest_keys.is_empty() {
                let mut all = vec![omit_key_type.clone()];
                all.extend(unspreadable_to_rest_keys);
                omit_key_type = self.get_union_type(all);
            }
            if omit_key_type.flags.intersects(TypeFlags::Never) {
                return source;
            }
            let omit_type_alias = match self.get_global_omit_symbol() {
                Some(a) => a,
                None => return self.error_type(),
            };
            return self.get_type_alias_instantiation(
                &omit_type_alias,
                &[source.clone(), omit_key_type],
                None,
            );
        }
        let mut members = SymbolTable::new();
        for prop in spreadable_properties {
            members.insert(prop.name.clone(), crate::checker::mig::m1e::r28k9_defs::get_spread_symbol(self, &prop, false));
        }
        let index_infos = self.get_index_infos_of_type(&source);
        let symbol_arg = match symbol {
            Some(s) => Arc::clone(s),
            None => self.unknown_symbol(),
        };
        let mut result = self.new_anonymous_type(&symbol_arg, members, vec![], vec![], index_infos);
        if let Some(result_mut) = Arc::get_mut(&mut result) {
            result_mut.object_flags |= ObjectFlags::ObjectRestType;
        }
        result
    }

    pub fn get_return_type_from_annotation(&mut self, declaration: &Arc<Node>) -> Arc<Type> {
        if is_constructor_declaration(declaration) {
            let parent = declaration
                .parent()
                .expect("constructor should have a parent");
            let parent_symbol = self.get_symbol_of_node(&parent).expect("parent symbol");
            let merged = self.get_merged_symbol(&parent_symbol);
            return self.get_declared_type_of_class_or_interface(&merged);
        }
        if let Some(return_type) = declaration.type_node() {
            return self.get_type_from_type_node(return_type);
        }
        if is_get_accessor_declaration(declaration) && self.has_bindable_name(declaration) {
            let set_accessor = self
                .get_symbol_of_declaration(declaration)
                .as_deref()
                .and_then(|s| get_declaration_of_kind(s, SyntaxKind::SetAccessor));
            if let Some(set_accessor) = set_accessor {
                if let Some(accessor_type) = self.get_annotated_accessor_type(&set_accessor) {
                    return accessor_type;
                }
            }
        }
        self.get_return_type_of_full_signature(declaration)
            .unwrap_or_else(|| self.error_type())
    }

    pub fn get_return_type_of_full_signature(&mut self, node: &Node) -> Option<Arc<Type>> {
        let signature = self.get_signature_of_full_signature_type(node)?;
        self.get_return_type_of_signature(&signature)
    }

    pub fn get_return_type_of_single_non_generic_signature(
        &mut self,
        func_type: &Arc<Type>,
        kind: SignatureKind,
    ) -> Option<Arc<Type>> {
        let signature = self.get_single_signature(func_type, kind, true);
        if let Some(signature) = signature {
            if signature.type_parameters.is_empty() {
                return self.get_return_type_of_signature(&signature);
            }
        }
        None
    }

    pub fn get_return_type_of_single_non_generic_signature_of_call_chain(
        &mut self,
        expr: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        let Some(expr_expression) = expr.expression() else {
            return None;
        };
        let func_type = self.check_expression_ex(expr_expression, CheckMode::Normal);
        let non_optional_type = self.get_optional_expression_type(&func_type, expr_expression);
        let return_type =
            self.get_return_type_of_single_non_generic_signature(&func_type, SignatureKind::Call)?;
        Some(self.propagate_optional_type_marker(
            &return_type,
            expr,
            non_optional_type.id != func_type.id,
        ))
    }

    pub fn get_siblings_of_context(&mut self, context: &mut WideningContext) -> Vec<Arc<Type>> {
        if context.siblings.is_none() {
            let mut siblings: Vec<Arc<Type>> = vec![];
            if let Some(parent) = &mut context.parent {
                for t in self.get_siblings_of_context(parent) {
                    if is_object_literal_type(&t) {
                        if let Some(prop) =
                            self.get_property_of_object_type(&t, &context.property_name)
                        {
                            let prop_type = self.get_type_of_symbol(&prop);
                            siblings.extend(prop_type.distributed());
                        }
                    }
                }
            }
            context.siblings = Some(siblings);
        }
        context.siblings.clone().unwrap_or_default()
    }

    pub fn get_signature_instantiation_without_filling_in_type_arguments(
        &mut self,
        sig: &Arc<Signature>,
        type_arguments: &[Arc<Type>],
    ) -> Arc<Signature> {
        let key = CachedSignatureKey {
            sig: sig.clone(),
            key: get_type_list_key(type_arguments),
        };
        if let Some(instantiation) = self.cached_signatures.get(&key) {
            return instantiation.clone();
        }
        let instantiation = self.create_signature_instantiation(sig, type_arguments);
        self.cached_signatures.insert(key, instantiation.clone());
        instantiation
    }

    pub fn get_signature_of_full_signature_type(&mut self, node: &Node) -> Option<Arc<Signature>> {
        if is_in_js_file(node)
            && (is_function_declaration(node)
                || is_method_declaration(node)
                || is_function_expression_or_arrow_function(node))
        {
            if let Some(full_signature) = function_like_data_full_signature(node) {
                let t = self.get_type_from_type_node(&full_signature);
                return self.get_single_call_signature(&t);
            }
        }
        None
    }

    pub fn get_signatures_of_structured_type(
        &mut self,
        t: &Arc<Type>,
        kind: SignatureKind,
    ) -> Vec<Arc<Signature>> {
        if !t.flags.intersects(TypeFlags::STRUCTURED_TYPE) {
            return vec![];
        }
        let resolved = self.resolve_structured_type_members(t);
        let structured = match resolved.as_structured() {
            Some(d) => d,
            None => return vec![],
        };
        if kind == SignatureKind::Call {
            structured.call_signatures().to_vec()
        } else {
            structured.construct_signatures().to_vec()
        }
    }

    pub fn get_signatures_of_symbol(&mut self, symbol: Option<&Arc<Symbol>>) -> Vec<Arc<Signature>> {
        let symbol = match symbol {
            Some(s) => s,
            None => return vec![],
        };
        let mut result: Vec<Arc<Signature>> = vec![];
        for (i, decl) in symbol.declarations.iter().enumerate() {
            if !is_function_like(decl) {
                continue;
            }
            if i > 0 && decl.body().is_some() {
                let previous = &symbol.declarations[i - 1];
                if decl.parent_rc().id() == previous.parent_rc().id()
                    && decl.kind == previous.kind
                    && (decl.pos() == previous.end()
                        || previous.flags.intersects(NodeFlags::Reparsed))
                {
                    continue;
                }
            }
            let mut sig = self.get_signature_of_full_signature_type(decl);
            if sig.is_none() {
                sig = self.get_signature_from_declaration(decl);
            }
            result.push(sig.unwrap());
        }
        result
    }

    pub fn get_single_call_or_construct_signature(&mut self, t: &Arc<Type>) -> Option<Arc<Signature>> {
        if let Some(call_sig) = self.get_single_signature(t, SignatureKind::Call, false) {
            return Some(call_sig);
        }
        self.get_single_signature(t, SignatureKind::Construct, false)
    }

    pub fn get_single_signature(
        &mut self,
        t: &Arc<Type>,
        kind: SignatureKind,
        allow_members: bool,
    ) -> Option<Arc<Signature>> {
        if t.flags.intersects(TypeFlags::Object) {
            let resolved = self.resolve_structured_type_members(t);
            let structured = resolved.as_structured()?;
            if allow_members || (structured.properties.is_empty() && structured.index_infos.is_empty())
            {
                let calls = structured.call_signatures();
                let constructs = structured.construct_signatures();
                if kind == SignatureKind::Call && calls.len() == 1 && constructs.is_empty() {
                    return Some(calls[0].clone());
                }
                if kind == SignatureKind::Construct && constructs.len() == 1 && calls.is_empty() {
                    return Some(constructs[0].clone());
                }
            }
        }
        None
    }

    pub fn get_spread_argument_index(&self, args: &[Arc<Node>]) -> i64 {
        find_index(args, is_spread_argument)
            .map(|i| i as i64)
            .unwrap_or(-1)
    }

    pub fn get_spread_indices(&mut self, node: &Node) -> (i64, i64) {
        let computed = self
            .array_literal_links
            .get(node)
            .map(|l| l.indices_computed)
            .unwrap_or(false);
        if !computed {
            let mut first = -1i32;
            let mut last = -1i32;
            for (i, element) in tsox_frontend::ast::mig::m3b::elements(node).iter().enumerate() {
                if is_spread_element(element) {
                    if first < 0 {
                        first = i as i32;
                    }
                    last = i as i32;
                }
            }
            if let Some(links) = self.array_literal_links.get_mut(node) {
                links.first_spread_index = first;
                links.last_spread_index = last;
                links.indices_computed = true;
            }
        }
        match self.array_literal_links.get(node) {
            Some(links) => (links.first_spread_index as i64, links.last_spread_index as i64),
            None => (-1, -1),
        }
    }

    pub fn get_string_mapping_type_for_generic_type(
        &mut self,
        symbol: &Arc<Symbol>,
        t: &Arc<Type>,
    ) -> Arc<Type> {
        let key = r24k9_defs::string_mapping_key_hash(symbol, t);
        if let Some(result) = self.string_mapping_types.get(&key) {
            return result.clone();
        }
        let result = self.new_string_mapping_type(symbol, t);
        self.string_mapping_types.insert(key, result.clone());
        result
    }

    pub fn get_substitution_intersection(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if self.is_no_infer_type(t) {
            return match &t.data {
                TypeData::Substitution(d) => d.base_type.clone().unwrap_or_else(|| t.clone()),
                _ => t.clone(),
            };
        }
        let (constraint, base_type) = match &t.data {
            TypeData::Substitution(d) => (
                d.constraint.clone().unwrap_or_else(|| t.clone()),
                d.base_type.clone().unwrap_or_else(|| t.clone()),
            ),
            _ => return t.clone(),
        };
        self.get_intersection_type(vec![constraint, base_type])
    }

    pub fn get_substitution_type(
        &mut self,
        base_type: &Arc<Type>,
        constraint: &Arc<Type>,
    ) -> Arc<Type> {
        if constraint.flags.intersects(TypeFlags::ANY_OR_UNKNOWN)
            || constraint.id == base_type.id
            || base_type.flags.intersects(TypeFlags::Any)
        {
            return base_type.clone();
        }
        self.get_or_create_substitution_type(base_type, constraint)
    }

    pub fn get_suggested_boolean_operator(&self, operator: SyntaxKind) -> SyntaxKind {
        match operator {
            SyntaxKind::BarToken | SyntaxKind::BarEqualsToken => SyntaxKind::BarBarToken,
            SyntaxKind::CaretToken | SyntaxKind::CaretEqualsToken => {
                SyntaxKind::ExclamationEqualsEqualsToken
            }
            SyntaxKind::AmpersandToken | SyntaxKind::AmpersandEqualsToken => {
                SyntaxKind::AmpersandAmpersandToken
            }
            _ => SyntaxKind::Unknown,
        }
    }
}
