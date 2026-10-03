#![allow(unused_imports)]
use crate::checker::mig::m2a::r19k11_defs::*;
use tsox_frontend::scanner::mig::m3i::is_identifier_text;
use tsox_frontend::ast::mig::w2::create_modifiers_from_modifier_flags;
use crate::checker::exports_union_reduction::get_declaration_modifier_flags_from_symbol;
use crate::checker::mig::m2a::r18k8_flags::ConstantValue;
use crate::checker::mig::wc3_2::is_const_enum_symbol;
use crate::checker::utilities_is_optional_symbol::is_numeric_literal_name;

use std::collections::HashSet;
use std::sync::Arc;

use tsox_frontend::ast::{
    is_class_like, is_enum_member, is_interface_declaration, is_module_declaration,
    is_private_identifier, Node, NodeData, NodeFlags, Symbol, SymbolFlags, SyntaxKind,
};

use crate::checker::mig::m2b::r22k6_defs::{
    node_modifier_flags, set_emit_flags_single_line, R22K6NodeBuilderExt, R22K6NodeFactoryExt,
    SignatureToSignatureDeclarationOptions,
};
use crate::checker::mig::m2f::r24k12_defs::symbol_format_flags_to_node_builder_flags;
use crate::checker::mig::m2g::r22k9_defs::NodeBuilderPseudoExt22;
use crate::checker::nodecopy_builder::NodeBuilderImpl;
use crate::checker::mig::m2f::{replace_modifiers, NodeFactoryExt};
use crate::checker::symboltracker::{NodeBuilderContext, NodeBuilderFlags};
use crate::checker::types::{
    SignatureKind, SignatureFlags, SymbolFormatFlags, Ternary, Type,
};

pub fn is_expanding(ctx: &NodeBuilderContext) -> bool { ::tsox_core::fntrace::enter("is_expanding"); 
    ctx.max_expansion_depth != -1
}

pub fn is_hash_private(s: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("is_hash_private"); 
    s.value_declaration
        .as_ref()
        .and_then(|d| d.name())
        .is_some_and(|n| is_private_identifier(&n))
}

pub fn type_elements_to_class_elements(
    f: &crate::checker::nodecopy_builder::NodeFactoryStub,
    members: Vec<Arc<Node>>,
) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("type_elements_to_class_elements"); 
    let mut members = members;
    for i in 0..members.len() {
        let m = &members[i];
        match m.kind {
            SyntaxKind::PropertySignature => {
                if let NodeData::PropertySignatureDeclaration(ps) = &m.data {
                    members[i] = f.new_property_declaration(
                        m.modifiers().cloned(),
                        ps.name.clone(),
                        ps.postfix_token.clone(),
                        Some(ps.type_node.clone()),
                        None,
                    );
                }
            }
            SyntaxKind::MethodSignature => {
                if let NodeData::MethodSignatureDeclaration(ms) = &m.data {
                    members[i] = f.new_method_declaration(
                        m.modifiers().cloned(),
                        None,
                        ms.name.clone(),
                        ms.postfix_token.clone(),
                        ms.type_parameters.clone(),
                        ms.parameters.clone(),
                        ms.type_node.clone(),
                        None,
                        None,
                    );
                }
            }
            _ => {}
        }
    }
    members
}

impl<'a> NodeBuilderImpl<'a> {
    pub fn expand_symbol_for_hover(&mut self, symbol: &Arc<Symbol>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("expand_symbol_for_hover"); 
        let mut results: Vec<Arc<Node>> = Vec::new();
        if symbol.flags.intersects(SymbolFlags::ENUM) {
            if let Some(node) = self.expand_enum_decl(symbol) {
                results.push(node);
            }
        }
        if symbol.flags.intersects(SymbolFlags::Class) {
            if let Some(node) = self.expand_class_decl(symbol) {
                results.push(node);
            }
        }
        if symbol
            .flags
            .intersects(SymbolFlags::ValueModule | SymbolFlags::NamespaceModule)
        {
            if let Some(node) = self.expand_module_decl(symbol) {
                results.push(node);
            }
        }
        if symbol.flags.intersects(SymbolFlags::Interface)
            && !symbol.flags.intersects(SymbolFlags::Class)
        {
            if let Some(node) = self.expand_interface_decl(symbol) {
                results.push(node);
            }
        }
        results
    }

    pub fn expand_enum_decl(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("expand_enum_decl"); 
        let ch = unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
        let name = symbol.name.clone();
        self.ctx.borrow_mut().approximate_length += 9 + name.len();
        let type_of_symbol = ch.get_type_of_symbol(symbol);
        let member_props: Vec<Arc<Symbol>> = ch
            .get_properties_of_type(&type_of_symbol)
            .into_iter()
            .filter(|p| p.flags.intersects(SymbolFlags::EnumMember))
            .collect();
        let mut members: Vec<Arc<Node>> = Vec::new();
        for (i, p) in member_props.iter().enumerate() {
            if self.check_truncation_length_if_expanding() && i + 3 < member_props.len() - 1 {
                self.ctx.borrow_mut().expansion_truncated = true;
                members.push(self.f.new_enum_member(
                    self.f.new_string_literal(
                        &format!(" ... {} more ... ", member_props.len() - i - 1),
                        0,
                    ),
                    None,
                ));
                let last = &member_props[member_props.len() - 1];
                let initializer = self.enum_member_initializer(last);
                members.push(self.f.new_enum_member(
                    self.f.new_identifier(&last.name),
                    initializer,
                ));
                break;
            }
            let member_decl = p
                .declarations
                .iter()
                .find(|d| is_enum_member(d))
                .cloned();
            let initializer = match member_decl.as_ref().and_then(|d| match &d.data {
                NodeData::EnumMember(m) => m.initializer.clone(),
                _ => None,
            }) {
                Some(orig) => Some(self.deep_clone_node(&orig)),
                None => self.enum_member_initializer(p),
            };
            self.ctx.borrow_mut().approximate_length += 4 + p.name.len();
            if initializer.is_some() {
                self.ctx.borrow_mut().approximate_length += 5;
            }
            members.push(self.f.new_enum_member(self.f.new_identifier(&p.name), initializer));
        }

        let mut const_modifier = tsox_frontend::ast::ModifierFlags::empty();
        if is_const_enum_symbol(symbol) {
            const_modifier = tsox_frontend::ast::ModifierFlags::Const;
        }
        let mods = if !const_modifier.is_empty() {
            self.f.new_modifier_list(
                tsox_frontend::ast::mig::w2::create_modifiers_from_modifier_flags(
                    const_modifier,
                    |k| self.f.new_modifier(k),
                ),
            )
        } else {
            None
        };
        Some(self.f.new_enum_declaration(
            mods,
            self.f.new_identifier(&name),
            self.f.new_node_list(members),
        ))
    }

    pub fn enum_member_initializer(&mut self, p: &Arc<Symbol>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("enum_member_initializer"); 
        let ch = unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
        let member_decl = p
            .declarations
            .iter()
            .find(|d| is_enum_member(d))
            .cloned()?;
        let val = ch.get_constant_value(&member_decl)?;
        Some(self.f.new_string_literal(&val, 0))
    }

    pub fn expand_class_decl(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("expand_class_decl"); 
        let name = symbol.name.clone();
        self.ctx.borrow_mut().approximate_length += 9 + name.len();

        let class_like_declarations: Vec<Arc<Node>> = symbol
            .declarations
            .iter()
            .filter(|d| is_class_like(d))
            .cloned()
            .collect();
        let original_decl = class_like_declarations.first().cloned();
        let old_enclosing = self.ctx.borrow().enclosing_declaration.clone();
        if let Some(ref od) = original_decl {
            self.ctx.borrow_mut().enclosing_declaration = Some(Arc::clone(od));
        }

        let result = self.expand_class_decl_worker(symbol, &name, &class_like_declarations, original_decl);

        self.ctx.borrow_mut().enclosing_declaration = old_enclosing;
        result
    }

    fn expand_class_decl_worker(
        &mut self,
        symbol: &Arc<Symbol>,
        name: &str,
        class_like_declarations: &[Arc<Node>],
        _original_decl: Option<Arc<Node>>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("expand_class_decl_worker"); 
        let ch = unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
        let local_params = ch
            .get_local_type_parameters_of_class_or_interface_or_type_alias(symbol);
        let mut type_param_decls: Vec<Arc<Node>> = Vec::with_capacity(local_params.len());
        for p in &local_params {
            type_param_decls.push(self.type_parameter_to_declaration(p));
        }

        let declared_type = ch.get_declared_type_of_class_or_interface(symbol);
        let class_type = ch.get_type_with_this_argument(&declared_type, None, false);
        let target_type = ch.get_target_type(&class_type);
        let base_types = ch.get_base_types(&target_type);
        let static_type = ch.get_type_of_symbol(symbol);
        let is_class = static_type
            .symbol
            .as_ref()
            .and_then(|s| s.value_declaration.as_ref())
            .is_some_and(|d| is_class_like(d));
        let static_base_type = if is_class {
            ch.get_base_constructor_type_of_class(&declared_type)
        } else {
            Some(ch.any_type())
        };

        let heritage_clauses = self.hover_heritage_clauses(class_like_declarations);

        let all_props = ch.get_properties_of_type(&class_type);
        let symbol_props = self.filter_inherited_properties(&class_type, &base_types, all_props);
        let public_props: Vec<Arc<Symbol>> = symbol_props
            .iter()
            .filter(|s| !is_hash_private(s))
            .cloned()
            .collect();
        let has_private = symbol_props.iter().any(|s| is_hash_private(s));

        let mut instance_members: Vec<Arc<Node>> = Vec::new();
        instance_members = self.serialize_properties_with_truncation(&public_props, instance_members);
        instance_members = type_elements_to_class_elements(&self.f, instance_members);
        instance_members = self.add_class_modifiers(instance_members, false);

        let static_props: Vec<Arc<Symbol>> = ch
            .get_properties_of_type(&static_type)
            .into_iter()
            .filter(|p| {
                !p.flags.intersects(SymbolFlags::Prototype)
                    && p.name != "prototype"
                    && !self.is_namespace_member(p)
            })
            .collect();
        let mut static_members: Vec<Arc<Node>> = Vec::new();
        static_members = self.serialize_properties_with_truncation(&static_props, static_members);
        static_members = type_elements_to_class_elements(&self.f, static_members);
        static_members = self.add_class_modifiers(static_members, true);

        let mut private_members: Vec<Arc<Node>> = Vec::new();
        if has_private {
            let hash_private: Vec<Arc<Symbol>> = symbol_props
                .iter()
                .filter(|s| is_hash_private(s))
                .cloned()
                .collect();
            private_members = self.serialize_properties_with_truncation(&hash_private, private_members);
            private_members = type_elements_to_class_elements(&self.f, private_members);
        }

        let constructors = self.serialize_constructors(
            &static_type,
            static_base_type.as_ref(),
            is_class,
            symbol,
        );

        let index_sigs = self.serialize_index_signatures_of_type(&class_type, base_types.first());

        let mut all_members: Vec<Arc<Node>> = Vec::with_capacity(
            index_sigs.len() + static_members.len() + constructors.len()
                + instance_members.len()
                + private_members.len(),
        );
        all_members.extend(index_sigs);
        all_members.extend(static_members);
        all_members.extend(constructors);
        all_members.extend(instance_members);
        all_members.extend(private_members);

        Some(self.f.new_class_declaration(
            None,
            self.f.new_identifier(name),
            self.f.new_node_list(type_param_decls),
            self.f.new_node_list(heritage_clauses),
            self.f.new_node_list(all_members),
        ))
    }

    pub fn add_class_modifiers(
        &mut self,
        members: Vec<Arc<Node>>,
        is_static: bool,
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("add_class_modifiers"); 
        let mut members = members;
        for i in 0..members.len() {
            let m = &members[i];
            let member_name = m.name();
            let member_symbol = member_name
                .as_ref()
                .and_then(|n| self.id_to_symbol.get(&n.id()).cloned());
            let Some(member_symbol) = member_symbol else {
                continue;
            };
            let mut mod_flags = get_declaration_modifier_flags_from_symbol(&member_symbol)
                - tsox_frontend::ast::ModifierFlags::Async;
            if is_static {
                mod_flags |= tsox_frontend::ast::ModifierFlags::Static;
            }
            if !mod_flags.is_empty() && tsox_frontend::ast::can_have_modifiers(m) {
                let existing = node_modifier_flags(m);
                if mod_flags != existing {
                    let new_mods = self.f.new_modifier_list(
                        create_modifiers_from_modifier_flags(
                            mod_flags | existing,
                            |k| self.f.new_modifier(k),
                        ),
                    );
                    members[i] = replace_modifiers(&self.f, m, new_mods);
                }
            }
        }
        members
    }

    pub fn expand_interface_decl(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("expand_interface_decl"); 
        let ch = unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
        let name = symbol.name.clone();
        self.ctx.borrow_mut().approximate_length += 14 + name.len();

        let interface_type = ch.get_declared_type_of_class_or_interface(symbol);
        let interface_declarations: Vec<Arc<Node>> = symbol
            .declarations
            .iter()
            .filter(|d| is_interface_declaration(d))
            .cloned()
            .collect();
        let local_params = ch
            .get_local_type_parameters_of_class_or_interface_or_type_alias(symbol);
        let mut type_param_decls: Vec<Arc<Node>> = Vec::with_capacity(local_params.len());
        for p in &local_params {
            type_param_decls.push(self.type_parameter_to_declaration(p));
        }
        let base_types = ch.get_base_types(&interface_type);
        let base_type = if !base_types.is_empty() {
            Some(ch.get_intersection_type(base_types.clone()))
        } else {
            None
        };

        let resolved = ch.resolve_structured_type_members(&interface_type);
        let mut members: Vec<Arc<Node>> = Vec::new();

        members.extend(self.serialize_index_signatures_of_type(&interface_type, base_type.as_ref()));
        for sig in ch.get_signatures_of_type(&interface_type, SignatureKind::Construct) {
            if sig.flags.intersects(SignatureFlags::Abstract) {
                continue;
            }
            members.push(
                self.signature_to_signature_declaration_helper(&sig, SyntaxKind::ConstructSignature, None),
            );
        }
        for sig in ch.get_signatures_of_type(&interface_type, SignatureKind::Call) {
            members.push(
                self.signature_to_signature_declaration_helper(&sig, SyntaxKind::CallSignature, None),
            );
        }
        let filtered_props = self.filter_inherited_properties(
            &interface_type,
            &base_types,
            ch.get_properties_of_type(&interface_type),
        );
        members = self.serialize_properties_with_truncation(&filtered_props, members);

        let heritage_clauses = self.hover_heritage_clauses(&interface_declarations);

        Some(self.f.new_interface_declaration(
            None,
            self.f.new_identifier(&name),
            self.f.new_node_list(type_param_decls),
            self.f.new_node_list(heritage_clauses),
            self.f.new_node_list(members),
        ))
    }

    pub fn hover_heritage_clauses(&mut self, declarations: &[Arc<Node>]) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("hover_heritage_clauses"); 
        let mut extends_types: Vec<Arc<Node>> = Vec::new();
        let mut implements_types: Vec<Arc<Node>> = Vec::new();
        for declaration in declarations {
            for heritage_element in tsox_frontend::ast::get_extends_heritage_clause_elements(declaration) {
                extends_types.push(self.deep_clone_node(&heritage_element));
            }
            for heritage_element in
                tsox_frontend::ast::get_implements_heritage_clause_elements(declaration)
            {
                implements_types.push(self.deep_clone_node(&heritage_element));
            }
        }

        let mut heritage_clauses: Vec<Arc<Node>> = Vec::new();
        if !extends_types.is_empty() {
            heritage_clauses.push(self.f.new_heritage_clause(
                SyntaxKind::ExtendsKeyword,
                self.f.new_node_list(extends_types),
            ));
        }
        if !implements_types.is_empty() {
            heritage_clauses.push(self.f.new_heritage_clause(
                SyntaxKind::ImplementsKeyword,
                self.f.new_node_list(implements_types),
            ));
        }
        heritage_clauses
    }

    pub fn serialize_properties_with_truncation(
        &mut self,
        properties: &[Arc<Symbol>],
        elements: Vec<Arc<Node>>,
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("serialize_properties_with_truncation"); 
        let properties: Vec<Arc<Symbol>> = properties
            .iter()
            .filter(|p| !p.flags.intersects(SymbolFlags::Prototype))
            .cloned()
            .collect();
        let mut elements = elements;
        for (i, p) in properties.iter().enumerate() {
            if self.check_truncation_length_if_expanding() && i + 3 < properties.len() - 1 {
                self.ctx.borrow_mut().expansion_truncated = true;
                let text = format!("... {} more ...", properties.len() - i - 1);
                elements.push(self.f.new_property_signature_declaration(
                    None,
                    self.f.new_identifier(&text),
                    None,
                    None,
                    None,
                ));
                let last = Arc::clone(&properties[properties.len() - 1]);
                elements = self.add_property_to_element_list(&last, elements);
                break;
            }
            elements = self.add_property_to_element_list(p, elements);
        }
        elements
    }

    pub fn serialize_constructors(
        &mut self,
        static_type: &Arc<Type>,
        static_base_type: Option<&Arc<Type>>,
        is_class: bool,
        symbol: &Arc<Symbol>,
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("serialize_constructors"); 
        let ch = unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
        let is_non_constructable = !is_class
            && symbol.value_declaration.is_some()
            && tsox_frontend::ast::is_in_js_file(symbol.value_declaration.as_ref().unwrap())
            && ch
                .get_signatures_of_type(static_type, SignatureKind::Construct)
                .is_empty();
        if is_non_constructable {
            self.ctx.borrow_mut().approximate_length += 21;
            let modifiers = create_modifiers_from_modifier_flags(
                tsox_frontend::ast::ModifierFlags::Private,
                |k| self.f.new_modifier(k),
            );
            return vec![self.f.new_constructor_declaration(
                self.f.new_modifier_list(modifiers),
                None,
                self.f.new_node_list(Vec::new()),
                None,
                None,
                None,
            )];
        }
        let signatures = ch.get_signatures_of_type(static_type, SignatureKind::Construct);
        if let Some(static_base_type) = static_base_type {
            let base_sigs = ch
                .get_signatures_of_type(static_base_type, SignatureKind::Construct);
            if base_sigs.is_empty()
                && signatures.iter().all(|sig| sig.parameters.is_empty())
            {
                return Vec::new();
            }
            if base_sigs.len() == signatures.len() {
                let mut all_match = true;
                for i in 0..base_sigs.len() {
                    if ch.compare_signatures_identical(
                        &signatures[i],
                        &base_sigs[i],
                        false,
                        false,
                        true,
                    ) != Ternary::True
                    {
                        all_match = false;
                        break;
                    }
                }
                if all_match {
                    return Vec::new();
                }
            }
            let mut private_protected = tsox_frontend::ast::ModifierFlags::empty();
            for sig in &signatures {
                if let Some(decl) = &sig.declaration {
                    private_protected |= node_modifier_flags(decl)
                        & (tsox_frontend::ast::ModifierFlags::Private
                            | tsox_frontend::ast::ModifierFlags::Protected);
                }
            }
            if !private_protected.is_empty() {
                let mods = self.f.new_modifier_list(
                    create_modifiers_from_modifier_flags(
                        private_protected,
                        |k| self.f.new_modifier(k),
                    ),
                );
                return vec![self.f.new_constructor_declaration(
                    mods,
                    None,
                    self.f.new_node_list(Vec::new()),
                    None,
                    None,
                    None,
                )];
            }
        } else if signatures.iter().all(|sig| sig.parameters.is_empty()) {
            return Vec::new();
        }
        let mut result: Vec<Arc<Node>> = Vec::new();
        for sig in &signatures {
            self.ctx.borrow_mut().approximate_length += 1;
            result.push(
                self.signature_to_signature_declaration_helper(sig, SyntaxKind::Constructor, None),
            );
        }
        result
    }

    pub fn serialize_index_signatures_of_type(
        &mut self,
        input: &Arc<Type>,
        base_type: Option<&Arc<Type>>,
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("serialize_index_signatures_of_type"); 
        let ch = unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
        let mut result: Vec<Arc<Node>> = Vec::new();
        for info in ch.get_index_infos_of_type(input) {
            if let Some(base_type) = base_type {
                let base_info = info
                    .key_type
                    .as_ref()
                    .and_then(|kt| ch.get_index_info_of_type(base_type, kt));
                if let Some(base_info) = base_info {
                    if let (Some(vt), Some(bvt)) = (&info.value_type, &base_info.value_type) {
                        if ch.is_type_identical_to(vt, bvt) {
                            continue;
                        }
                    }
                }
            }
            result.push(self.index_info_to_index_signature_declaration_helper(&info, None).unwrap());
        }
        result
    }

    pub fn serialize_namespace_member(
        &mut self,
        resolved: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("serialize_namespace_member"); 
        let ch = unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
        if resolved.flags.intersects(SymbolFlags::TypeAlias) {
            return self.serialize_type_alias_for_namespace(resolved, name);
        }
        if resolved.flags.intersects(SymbolFlags::ENUM) {
            return self.expand_enum_decl(resolved);
        }
        if resolved.flags.intersects(SymbolFlags::Class) {
            return self.expand_class_decl(resolved);
        }
        if resolved.flags.intersects(SymbolFlags::Interface) {
            return self.expand_interface_decl(resolved);
        }
        if resolved
            .flags
            .intersects(SymbolFlags::ValueModule | SymbolFlags::NamespaceModule)
        {
            return self.expand_module_decl(resolved);
        }
        let t = ch.get_type_of_symbol(resolved);
        let t = ch.get_widened_type(&t);
        self.ctx.borrow_mut().approximate_length += name.len() + 5;
        let ty_node = Some(self.serialize_type_for_declaration(None, &t, Some(resolved), true));
        Some(self.f.new_variable_statement(
            None,
            self.f.new_variable_declaration_list(
                self.f.new_node_list(vec![self.f.new_variable_declaration(
                    self.f.new_identifier(name),
                    None,
                    ty_node,
                    None,
                )]),
                NodeFlags::Let,
            ),
        ))
    }

    pub fn expand_module_decl(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("expand_module_decl"); 
        let ch = unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
        let exports = ch.get_exports_of_symbol(symbol);
        let mut members: Vec<Arc<Symbol>> = Vec::new();
        for sym in exports.entries.values() {
            if !self.is_namespace_member(sym) {
                continue;
            }
            if !is_identifier_text(&sym.name, tsox_frontend::ast::LanguageVariant::Standard) {
                continue;
            }
            members.push(Arc::clone(sym));
        }
        ch.sort_symbols(&mut members);
        self.ctx.borrow_mut().approximate_length += 14;

        let old_flags = self.ctx.borrow().flags;
        self.ctx.borrow_mut().flags |=
            NodeBuilderFlags::WriteTypeParametersInQualifiedName | symbol_format_flags_to_node_builder_flags(SymbolFormatFlags::UseOnlyExternalAliasing);
        let local_name = self.symbol_to_node(symbol, SymbolFlags::all());
        self.ctx.borrow_mut().flags = old_flags;

        struct HoverStatement {
            node: Arc<Node>,
            is_local: bool,
        }
        let mut body_stmts: Vec<HoverStatement> = Vec::new();
        let mut emitted_locals: HashSet<u64> = HashSet::new();
        let mut i = 0;
        while i < members.len() {
            let m = &members[i];
            if self.check_truncation_length_if_expanding() && i + 3 < members.len() - 1 {
                self.ctx.borrow_mut().expansion_truncated = true;
                body_stmts.push(HoverStatement {
                    node: self
                        .f
                        .new_expression_statement(self.f.new_identifier(&format!("... ({} more) ...", members.len() - i - 1))),
                    is_local: false,
                });
                i = members.len() - 1;
                i += 1;
                continue;
            }

            if m.flags.intersects(SymbolFlags::Alias) {
                let alias_decl = ch.get_declaration_of_alias_symbol(m);
                let target = alias_decl
                    .and_then(|d| ch.get_target_of_alias_declaration(&d))
                    .map(|t| ch.get_merged_symbol(&t));
                if let Some(target) = target {
                    if target.flags.intersects(
                        SymbolFlags::BlockScopedVariable
                            | SymbolFlags::FunctionScopedVariable
                            | SymbolFlags::Property,
                    ) && emitted_locals.insert(target.id())
                    {
                        let local_type = ch.get_type_of_symbol(&target);
                        let local_type = ch.get_widened_type(&local_type);
                        self.ctx.borrow_mut().approximate_length += target.name.len() + 5;
                        let ty_node = Some(self.serialize_type_for_declaration(
                            None,
                            &local_type,
                            Some(&target),
                            true,
                        ));
                        let local_stmt = self.f.new_variable_statement(
                            None,
                            self.f.new_variable_declaration_list(
                                self.f.new_node_list(vec![self.f.new_variable_declaration(
                                    self.f.new_identifier(&target.name),
                                    None,
                                    ty_node,
                                    None,
                                )]),
                                NodeFlags::Let,
                            ),
                        );
                        body_stmts.push(HoverStatement {
                            node: local_stmt,
                            is_local: true,
                        });
                    }
                    let target_name = target.name.clone();
                    self.ctx.borrow_mut().approximate_length += 16 + m.name.len();
                    let property_name = if m.name != target_name {
                        Some(self.f.new_identifier(&target_name))
                    } else {
                        None
                    };
                    let stmt = self.f.new_export_declaration(
                        None,
                        false,
                        self.f.new_named_exports(self.f.new_node_list(vec![self
                            .f
                            .new_export_specifier(
                                false,
                                property_name,
                                self.f.new_identifier(&m.name),
                            )])),
                        None,
                        None,
                    );
                    body_stmts.push(HoverStatement {
                        node: stmt,
                        is_local: false,
                    });
                    i += 1;
                    continue;
                }
            }

            let resolved = ch.resolve_symbol(m);

            if resolved
                .flags
                .intersects(SymbolFlags::Function | SymbolFlags::Method)
            {
                let t = ch.get_type_of_symbol(&resolved);
                let sigs = ch.get_signatures_of_type(&t, SignatureKind::Call);
                for sig in &sigs {
                    self.ctx.borrow_mut().approximate_length += 1;
                    let decl = self.signature_to_signature_declaration_helper(
                        sig,
                        SyntaxKind::FunctionDeclaration,
                        Some(&SignatureToSignatureDeclarationOptions {
                            name: self.f.new_identifier(&m.name),
                        }),
                    );
                    body_stmts.push(HoverStatement {
                        node: decl,
                        is_local: false,
                    });
                }
                let merged = ch.get_merged_symbol(&resolved);
                let has_module_exports = merged
                    .flags
                    .intersects(SymbolFlags::ValueModule | SymbolFlags::NamespaceModule)
                    && !merged.exports.is_empty();
                if !has_module_exports {
                    body_stmts.push(HoverStatement {
                        node: self.f.new_module_declaration(
                            None,
                            SyntaxKind::NamespaceKeyword,
                            self.f.new_identifier(&m.name),
                            None,
                            self.f.new_module_block(self.f.new_node_list(Vec::new())),
                        ),
                        is_local: false,
                    });
                }
                i += 1;
                continue;
            }

            if let Some(node) = self.serialize_namespace_member(&resolved, &m.name) {
                body_stmts.push(HoverStatement {
                    node,
                    is_local: false,
                });
            }
            i += 1;
        }

        for s in body_stmts.iter_mut() {
            if s.is_local || tsox_frontend::ast::is_export_declaration(&s.node) {
                continue;
            }
            if tsox_frontend::ast::can_have_modifiers(&s.node) {
                let mf = node_modifier_flags(&s.node) | tsox_frontend::ast::ModifierFlags::Export;
                let new_mods = self.f.new_modifier_list(
                    create_modifiers_from_modifier_flags(mf, |k| {
                        self.f.new_modifier(k)
                    }),
                );
                s.node = replace_modifiers(&self.f, &s.node, new_mods);
            }
        }

        let mut body_statements: Vec<Arc<Node>> = Vec::with_capacity(body_stmts.len());
        for s in &body_stmts {
            body_statements.push(Arc::clone(&s.node));
        }
        let all_exported = !body_statements.is_empty()
            && body_statements.iter().all(|d| {
                tsox_frontend::ast::has_syntactic_modifier(d, tsox_frontend::ast::ModifierFlags::Export)
            });
        if all_exported {
            for stmt in body_statements.iter_mut() {
                if tsox_frontend::ast::can_have_modifiers(stmt) {
                    let mf = node_modifier_flags(stmt) - tsox_frontend::ast::ModifierFlags::Export;
                    let new_mods = self.f.new_modifier_list(
                        create_modifiers_from_modifier_flags(mf, |k| {
                            self.f.new_modifier(k)
                        }),
                    );
                    *stmt = replace_modifiers(&self.f, stmt, new_mods);
                }
            }
        }

        let mut keyword = SyntaxKind::NamespaceKeyword;
        if !tsox_frontend::ast::is_identifier(&local_name) {
            keyword = SyntaxKind::ModuleKeyword;
        }
        let mut attributes: Option<Arc<Node>> = None;
        if let Some(declaration) = symbol.declarations.iter().find(|d| {
            is_module_declaration(d)
                && match &d.data {
                    NodeData::ModuleDeclaration(md) => md.attributes.is_some(),
                    _ => false,
                }
        }) {
            if let NodeData::ModuleDeclaration(md) = &declaration.data {
                if let Some(ref attrs) = md.attributes {
                    attributes = Some(self.deep_clone_node(attrs));
                }
            }
        }
        if let Some(ref attrs) = attributes {
            set_emit_flags_single_line(&self.e, attrs);
        }
        Some(self.f.new_module_declaration(
            None,
            keyword,
            local_name,
            attributes,
            self.f.new_module_block(self.f.new_node_list(body_statements)),
        ))
    }

    pub fn serialize_type_alias_for_namespace(
        &mut self,
        symbol: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("serialize_type_alias_for_namespace"); 
        let ch = unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
        let alias_type = ch.get_declared_type_of_type_alias(symbol);
        let type_params = ch
            .get_local_type_parameters_of_class_or_interface_or_type_alias(symbol);
        let mut type_param_decls: Vec<Arc<Node>> = Vec::with_capacity(type_params.len());
        for p in &type_params {
            type_param_decls.push(self.type_parameter_to_declaration(p));
        }
        let restore_flags = self.save_restore_flags();
        self.ctx.borrow_mut().flags |= NodeBuilderFlags::InTypeAlias;
        let type_node = self.type_to_type_node(&alias_type);
        restore_flags.restore();
        self.ctx.borrow_mut().approximate_length += 8 + name.len();
        Some(self.f.new_type_alias_declaration(
            None,
            self.f.new_identifier(name),
            self.f.new_node_list(type_param_decls),
            type_node,
        ))
    }

    pub fn filter_inherited_properties(
        &mut self,
        t: &Arc<Type>,
        base_types: &[Arc<Type>],
        properties: Vec<Arc<Symbol>>,
    ) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("filter_inherited_properties"); 
        let ch = unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
        if base_types.is_empty() {
            return properties;
        }
        let mut props_by_name: std::collections::HashMap<&str, &Arc<Symbol>> =
            std::collections::HashMap::with_capacity(properties.len());
        for p in &properties {
            props_by_name.insert(p.name.as_str(), p);
        }
        let mut inherited: HashSet<String> = HashSet::new();
        let target = ch.get_target_type(t);
        let this_type = target
            .as_interface_type()
            .and_then(|i| i.this_type.clone())
            .expect("interface this_type");
        for base in base_types {
            let base_with_this = ch
                .get_type_with_this_argument(base, Some(&this_type), false);
            for prop in ch.get_properties_of_type(&base_with_this) {
                if let Some(existing) = props_by_name.get(prop.name.as_str()) {
                    match (prop.parent(), existing.parent()) {
                        (Some(pp), Some(ep)) if Arc::ptr_eq(&pp, &ep) => {
                            inherited.insert(prop.name.clone());
                        }
                        (None, None) => {
                            inherited.insert(prop.name.clone());
                        }
                        _ => {}
                    }
                }
            }
        }
        if inherited.is_empty() {
            return properties;
        }
        properties
            .into_iter()
            .filter(|p| !inherited.contains(&p.name))
            .collect()
    }

    pub fn is_namespace_member(&mut self, p: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("is_namespace_member"); 
        p.flags.intersects(SymbolFlags::TYPE | SymbolFlags::NAMESPACE | SymbolFlags::Alias)
            || !((p.flags.intersects(SymbolFlags::Prototype))
                || p.name == "prototype"
                || (p.value_declaration.as_ref().is_some_and(|vd| {
                    tsox_frontend::ast::has_static_modifier(vd)
                        && vd
                            .parent()
                            .as_ref()
                            .is_some_and(|par| is_class_like(par))
                })))
    }
}
