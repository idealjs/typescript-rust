#![allow(unused_imports)]

use crate::checker::utilities::*;

pub(crate) use crate::checker::utilities_token_is_identifier_or_keyword::AssignmentKind;

pub fn has_only_expression_initialization(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::VariableDeclaration
            | SyntaxKind::Parameter
            | SyntaxKind::BindingElement
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertyAssignment
            | SyntaxKind::EnumMember
    )
}

pub fn is_super_call(n: &Node) -> bool {
    tsox_frontend::ast::is_call_expression(n)
        && n.expression()
            .map(|e| e.kind == SyntaxKind::SuperKeyword)
            .unwrap_or(false)
}

pub fn is_call_chain(node: &Node) -> bool {
    tsox_frontend::ast::is_call_expression(node) && node.flags.contains(NodeFlags::OptionalChain)
}

pub fn is_non_null_access(node: &Node) -> bool {
    tsox_frontend::ast::is_access_expression(node)
        && node
            .expression()
            .map(|e| tsox_frontend::ast::is_non_null_expression(e))
            .unwrap_or(false)
}

pub fn is_this_property(node: &Node) -> bool {
    (tsox_frontend::ast::is_property_access_expression(node)
        || tsox_frontend::ast::is_element_access_expression(node))
        && node
            .expression()
            .map(|e| e.kind == SyntaxKind::ThisKeyword)
            .unwrap_or(false)
}

pub fn is_optional_declaration(declaration: &Node) -> bool {
    tsox_frontend::ast::is_question_token(
        tsox_frontend::ast::mig::m3c::question_token(declaration).map(Arc::as_ref),
    )
}

pub fn is_type_assertion(node: &Node) -> bool {
    tsox_frontend::ast::is_assertion_expression(node)
}

pub fn is_empty_object_literal(expression: &Node) -> bool {
    tsox_frontend::ast::is_object_literal_expression(expression)
}

pub fn is_empty_array_literal(expression: &Node) -> bool {
    tsox_frontend::ast::is_array_literal_expression(expression)
}

pub fn has_type(node: &Node) -> bool {
    node.type_node().is_some()
}

pub fn can_have_flow_node(node: &Node) -> bool {
    let _ = node;
    false
}

pub fn is_private_identifier_symbol(symbol: &Symbol) -> bool {
    symbol.name.starts_with(&format!(
        "{}#",
        tsox_frontend::ast::INTERNAL_SYMBOL_NAME_PREFIX
    ))
}

pub fn is_known_symbol(symbol: &Symbol) -> bool {
    is_late_bound_name(&symbol.name)
}

pub fn is_external_module_symbol(module_symbol: &Symbol) -> bool {
    module_symbol.flags.contains(SymbolFlags::MODULE) && module_symbol.name.starts_with('"')
}

pub fn has_export_assignment_symbol(module_symbol: &Symbol) -> bool {
    module_symbol
        .exports
        .get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
        .is_some()
}

pub fn is_static_private_identifier_property(s: &Symbol) -> bool {
    s.value_declaration
        .as_ref()
        .map(|d| tsox_frontend::ast::is_static(d))
        .unwrap_or(false)
}

pub fn get_declarations_of_kind(symbol: &Symbol, kind: SyntaxKind) -> Vec<Arc<Node>> {
    symbol
        .declarations
        .iter()
        .filter(|d| d.kind == kind)
        .cloned()
        .collect()
}

pub fn all_declarations_in_same_source_file(symbol: &Symbol) -> bool {
    if symbol.declarations.len() > 1 {
        let mut source_file_id: Option<u64> = None;
        for (i, d) in symbol.declarations.iter().enumerate() {
            if let Some(sf) = tsox_frontend::ast::get_source_file_of_node(d) {
                if i == 0 {
                    source_file_id = Some(sf.id());
                } else if source_file_id != Some(sf.id()) {
                    return false;
                }
            }
        }
    }
    true
}

pub fn get_index_symbol_from_symbol_table(symbol_table: &SymbolTable) -> Option<Arc<Symbol>> {
    symbol_table
        .get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_INDEX)
        .cloned()
}

pub fn symbols_to_array(symbols: &SymbolTable) -> Vec<Arc<Symbol>> {
    symbols
        .iter()
        .filter(|(id, _)| !is_reserved_member_name(id))
        .map(|(_, symbol)| Arc::clone(symbol))
        .collect()
}

pub fn create_symbol_table(symbols: &[Arc<Symbol>]) -> SymbolTable {
    let mut result = SymbolTable::new();
    for symbol in symbols {
        result.insert(symbol.name.clone(), Arc::clone(symbol));
    }
    result
}

pub fn is_object_or_array_literal_type(t: &Type) -> bool {
    t.object_flags
        .intersects(ObjectFlags::ObjectLiteral | ObjectFlags::ArrayLiteral)
}

pub fn is_this_type_parameter(t: &Type) -> bool {
    t.flags.contains(TypeFlags::TypeParameter)
        && matches!(&t.data, TypeData::TypeParameter(tp) if tp.is_this_type)
}

pub fn get_type_name_symbol(t: &Type) -> Option<Arc<Symbol>> {
    if let Some(alias) = &t.alias {
        return alias.symbol.clone();
    }
    if t.flags
        .intersects(TypeFlags::TypeParameter | TypeFlags::StringMapping)
        || t.object_flags
            .intersects(OBJECT_FLAGS_CLASS_OR_INTERFACE | ObjectFlags::Reference)
    {
        return t.symbol.clone();
    }
    None
}

pub fn get_object_type_name(t: &Type) -> Option<Arc<Symbol>> {
    if t.object_flags
        .intersects(OBJECT_FLAGS_CLASS_OR_INTERFACE | ObjectFlags::Reference)
    {
        return t.symbol.clone();
    }
    None
}

pub fn get_sort_order_flags(t: &Type) -> u32 {
    if t.flags.intersects(TypeFlags::EnumLiteral | TypeFlags::Enum)
        && !t.flags.contains(TypeFlags::Union)
    {
        return TypeFlags::Enum.bits();
    }
    t.flags.bits()
}

pub fn compare_type_names(t1: &Type, t2: &Type) -> std::cmp::Ordering {
    let s1 = get_type_name_symbol(t1);
    let s2 = get_type_name_symbol(t2);
    if s1.as_ref().map(|s| s.id()) == s2.as_ref().map(|s| s.id()) {
        if let Some(alias) = &t1.alias {
            return compare_type_lists(
                &alias.type_arguments,
                &t2.alias.as_ref().unwrap().type_arguments,
            );
        }
        return std::cmp::Ordering::Equal;
    }
    match (s1, s2) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(_), None) => std::cmp::Ordering::Less,
        (Some(a), Some(b)) => a.name.cmp(&b.name),
    }
}

pub fn compare_type_lists(s1: &[Arc<Type>], s2: &[Arc<Type>]) -> std::cmp::Ordering {
    if s1.len() != s2.len() {
        return s1.len().cmp(&s2.len());
    }
    for (t1, t2) in s1.iter().zip(s2.iter()) {
        let c = compare_types(t1, t2);
        if c != std::cmp::Ordering::Equal {
            return c;
        }
    }
    std::cmp::Ordering::Equal
}

pub fn type_parameters_match(a: &Type, b: &Type) -> bool {
    // 恒等实例（含驻留的延迟 IndexedAccess T[P]）总是匹配
    if std::ptr::eq(a as *const Type, b as *const Type) || a.id == b.id {
        return true;
    }
    // 延迟 IndexedAccess 按成分结构等价（Go 依赖完整驻留，我们补结构判定）
    if let (TypeData::IndexedAccess(x), TypeData::IndexedAccess(y)) = (&a.data, &b.data) {
        let obj_eq = x
            .object_type
            .as_ref()
            .zip(y.object_type.as_ref())
            .is_some_and(|(p, q)| p.id == q.id);
        let idx_eq = x
            .index_type
            .as_ref()
            .zip(y.index_type.as_ref())
            .is_some_and(|(p, q)| p.id == q.id);
        if obj_eq && idx_eq {
            return true;
        }
    }
    if !a.flags.contains(TypeFlags::TypeParameter) || !b.flags.contains(TypeFlags::TypeParameter) {
        return false;
    }
    match (&a.symbol, &b.symbol) {
        (Some(x), Some(y)) => Arc::ptr_eq(x, y),
        _ => false,
    }
}

pub fn compare_union_members(t1: &Type, t2: &Type) -> std::cmp::Ordering {
    get_sort_order_flags(t1)
        .cmp(&get_sort_order_flags(t2))
        .then_with(|| compare_type_names(t1, t2))
        .then_with(|| {
            let structural = compare_union_member_structure(t1, t2);
            if structural != std::cmp::Ordering::Equal {
                return structural;
            }
            const RESIDENT_LITERALS: TypeFlags = TypeFlags::from_bits_truncate(
                TypeFlags::StringLiteral.bits()
                    | TypeFlags::NumberLiteral.bits()
                    | TypeFlags::BigIntLiteral.bits()
                    | TypeFlags::BooleanLiteral.bits()
                    | TypeFlags::UniqueESSymbol.bits(),
            );
            if t1.flags.intersects(RESIDENT_LITERALS) || t2.flags.intersects(RESIDENT_LITERALS) {
                std::cmp::Ordering::Equal
            } else {
                t1.id.cmp(&t2.id)
            }
        })
}

fn compare_union_member_structure(t1: &Type, t2: &Type) -> std::cmp::Ordering {
    if t1.id == t2.id {
        return std::cmp::Ordering::Equal;
    }
    match (&t1.data, &t2.data) {
        (TypeData::Substitution(s1), TypeData::Substitution(s2)) => {
            compare_sub_terms(&s1.base_type, &s2.base_type)
                .then_with(|| compare_sub_terms(&s1.constraint, &s2.constraint))
        }
        _ if t1.flags.contains(TypeFlags::Object) && t2.flags.contains(TypeFlags::Object) => {
            match instantiated_signature_first(t1, t2) {
                Some(ordering) => ordering,
                None => std::cmp::Ordering::Equal,
            }
        }
        _ => std::cmp::Ordering::Equal,
    }
}

fn compare_sub_terms(
    s1: &Option<Arc<Type>>,
    s2: &Option<Arc<Type>>,
) -> std::cmp::Ordering {
    match (s1, s2) {
        (Some(a), Some(b)) => compare_union_members(a, b),
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(_), None) => std::cmp::Ordering::Greater,
    }
}

fn instantiated_signature_first(t1: &Type, t2: &Type) -> Option<std::cmp::Ordering> {
    let product_with_sigs = |t: &Type| {
        let is_product = t.object_flags.contains(ObjectFlags::Instantiated)
            || matches!(&t.data, TypeData::Object(o) if o.mapper.is_some());
        is_product
            && t.as_structured()
                .is_some_and(|s| !s.signatures.is_empty())
    };
    let plain_without_sigs = |t: &Type| {
        let is_product = t.object_flags.contains(ObjectFlags::Instantiated)
            || matches!(&t.data, TypeData::Object(o) if o.mapper.is_some());
        !is_product
            && t.as_structured()
                .is_none_or(|s| s.signatures.is_empty())
    };
    if product_with_sigs(t1) && plain_without_sigs(t2) {
        Some(std::cmp::Ordering::Less)
    } else if product_with_sigs(t2) && plain_without_sigs(t1) {
        Some(std::cmp::Ordering::Greater)
    } else {
        None
    }
}

pub fn compare_types(t1: &Type, t2: &Type) -> std::cmp::Ordering {
    if t1.id == t2.id {
        return std::cmp::Ordering::Equal;
    }
    let c = get_sort_order_flags(t1).cmp(&get_sort_order_flags(t2));
    if c != std::cmp::Ordering::Equal {
        return c;
    }
    let c = compare_type_names(t1, t2);
    if c != std::cmp::Ordering::Equal {
        return c;
    }
    let c = compare_type_data(t1, t2);
    if c != std::cmp::Ordering::Equal {
        return c;
    }

    t1.id.cmp(&t2.id)
}

fn compare_type_data(t1: &Type, t2: &Type) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let flags_only_ids = TypeFlags::Any
        .union(TypeFlags::Unknown)
        .union(TypeFlags::String)
        .union(TypeFlags::Number)
        .union(TypeFlags::Boolean)
        .union(TypeFlags::BigInt)
        .union(TypeFlags::ESSymbol)
        .union(TypeFlags::Void)
        .union(TypeFlags::Undefined)
        .union(TypeFlags::Null)
        .union(TypeFlags::Never)
        .union(TypeFlags::NonPrimitive);
    if t1.flags.intersects(flags_only_ids) {
        return Ordering::Equal;
    }
    match (&t1.data, &t2.data) {
        (TypeData::Union(u1), TypeData::Union(u2)) => {
            match (&u1.origin, &u2.origin) {
                (None, None) => {}
                (None, Some(_)) => return Ordering::Greater,
                (Some(_), None) => return Ordering::Less,
                (Some(o1), Some(o2)) => {
                    let c = compare_types(o1, o2);
                    if c != Ordering::Equal {
                        return c;
                    }
                }
            }
            compare_type_lists(&u1.union_or_intersection.types, &u2.union_or_intersection.types)
        }
        (TypeData::Intersection(i1), TypeData::Intersection(i2)) => compare_type_lists(
            &i1.union_or_intersection.types,
            &i2.union_or_intersection.types,
        ),
        (TypeData::Literal(l1), TypeData::Literal(l2)) => {
            if t1.flags.contains(TypeFlags::StringLiteral)
                && t2.flags.contains(TypeFlags::StringLiteral)
            {
                match (&l1.value, &l2.value) {
                    (LiteralValue::String(a), LiteralValue::String(b)) => a.cmp(b),
                    _ => Ordering::Equal,
                }
            } else if t1.flags.contains(TypeFlags::NumberLiteral)
                && t2.flags.contains(TypeFlags::NumberLiteral)
            {
                match (&l1.value, &l2.value) {
                    (LiteralValue::Number(a), LiteralValue::Number(b)) => a.0.total_cmp(&b.0),
                    _ => Ordering::Equal,
                }
            } else if t1.flags.contains(TypeFlags::BooleanLiteral)
                && t2.flags.contains(TypeFlags::BooleanLiteral)
            {
                match (&l1.value, &l2.value) {
                    (LiteralValue::Boolean(a), LiteralValue::Boolean(b)) => a.cmp(b),
                    _ => Ordering::Equal,
                }
            } else {
                Ordering::Equal
            }
        }
        (TypeData::Index(i1), TypeData::Index(i2)) => {
            let c = compare_option_types(i1.target.as_ref(), i2.target.as_ref());
            if c != Ordering::Equal {
                return c;
            }
            i1.index_flags.bits().cmp(&i2.index_flags.bits())
        }
        (TypeData::IndexedAccess(i1), TypeData::IndexedAccess(i2)) => {
            let c = compare_option_types(i1.object_type.as_ref(), i2.object_type.as_ref());
            if c != Ordering::Equal {
                return c;
            }
            compare_option_types(i1.index_type.as_ref(), i2.index_type.as_ref())
        }
        (TypeData::Substitution(s1), TypeData::Substitution(s2)) => {
            let c = compare_option_types(s1.base_type.as_ref(), s2.base_type.as_ref());
            if c != Ordering::Equal {
                return c;
            }
            compare_option_types(s1.constraint.as_ref(), s2.constraint.as_ref())
        }
        (TypeData::TemplateLiteral(t1), TypeData::TemplateLiteral(t2)) => {
            let c = t1.texts.cmp(&t2.texts);
            if c != Ordering::Equal {
                return c;
            }
            compare_type_lists(&t1.types, &t2.types)
        }
        (TypeData::StringMapping(s1), TypeData::StringMapping(s2)) => {
            compare_option_types(s1.target.as_ref(), s2.target.as_ref())
        }
        _ => Ordering::Equal,
    }
}

fn compare_option_types(t1: Option<&Arc<Type>>, t2: Option<&Arc<Type>>) -> std::cmp::Ordering {
    match (t1, t2) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(_), None) => std::cmp::Ordering::Less,
        (Some(a), Some(b)) => compare_types(a, b),
    }
}

pub fn get_assignment_target_kind(node: &Arc<Node>) -> AssignmentKind {
    let Some(target) = get_assignment_target(node) else {
        return AssignmentKind::None;
    };
    match &target.data {
        tsox_frontend::ast::NodeData::BinaryExpression(bin) => {
            if matches!(
                bin.operator_token.kind,
                SyntaxKind::EqualsToken
                    | SyntaxKind::AmpersandAmpersandEqualsToken
                    | SyntaxKind::BarBarEqualsToken
                    | SyntaxKind::QuestionQuestionEqualsToken
            ) {
                AssignmentKind::Definite
            } else {
                AssignmentKind::Compound
            }
        }
        tsox_frontend::ast::NodeData::PrefixUnaryExpression(_)
        | tsox_frontend::ast::NodeData::PostfixUnaryExpression(_) => AssignmentKind::Compound,
        tsox_frontend::ast::NodeData::ForInOrOfStatement(_) => AssignmentKind::Definite,
        _ => AssignmentKind::None,
    }
}

/// Go isConstTypeReference：`<const>expr` 的断言类型是无实参的
/// `const` 标识符引用（parseIdentifierName 接受关键字作类型名）
pub fn is_const_type_reference(type_node: &Node) -> bool {
    if type_node.kind != tsox_frontend::ast::SyntaxKind::TypeReference {
        return false;
    }
    if let tsox_frontend::ast::NodeData::TypeReferenceNode(d) = &type_node.data {
        if d.type_arguments.is_some() {
            return false;
        }
        if let tsox_frontend::ast::NodeData::Identifier(id) = &d.type_name.data {
            return id.text == "const";
        }
    }
    false
}

impl crate::checker::checker_checker_checker::Checker {
    pub fn compare_types_ordered(&self, t1: &Type, t2: &Type) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        if t1.id == t2.id {
            return Ordering::Equal;
        }
        let c = get_sort_order_flags(t1).cmp(&get_sort_order_flags(t2));
        if c != Ordering::Equal {
            return c;
        }
        let c = compare_type_names(t1, t2);
        if c != Ordering::Equal {
            return c;
        }
        let enum_like = TypeFlags::EnumLiteral.union(TypeFlags::Enum).union(TypeFlags::UniqueESSymbol);
        if t1.flags.intersects(enum_like) && t2.flags.intersects(enum_like) {
            let c = self.compare_symbols_worker(t1.symbol.as_ref(), t2.symbol.as_ref());
            if c != Ordering::Equal {
                return c;
            }
        } else if t1.flags.contains(TypeFlags::Object) && t2.flags.contains(TypeFlags::Object) {
            let c = self.compare_object_type_order(t1, t2);
            if c != Ordering::Equal {
                return c;
            }
        } else if t1.flags.contains(TypeFlags::TypeParameter) && t2.flags.contains(TypeFlags::TypeParameter) {
            let c = self.compare_symbols_worker(t1.symbol.as_ref(), t2.symbol.as_ref());
            if c != Ordering::Equal {
                return c;
            }
        } else if t1.flags.contains(TypeFlags::Conditional) && t2.flags.contains(TypeFlags::Conditional) {
            if let (TypeData::Conditional(c1), TypeData::Conditional(c2)) = (&t1.data, &t2.data) {
                let node1 = c1.root.as_ref().and_then(|r| r.node.as_ref());
                let node2 = c2.root.as_ref().and_then(|r| r.node.as_ref());
                let c = self.compare_nodes(node1, node2);
                if c != Ordering::Equal {
                    return c;
                }
                let c = crate::checker::mig::m3a_2::compare_type_mappers(
                    c1.mapper.as_ref(),
                    c2.mapper.as_ref(),
                );
                if c != Ordering::Equal {
                    return c;
                }
            }
        }
        let c = compare_type_data(t1, t2);
        if c != Ordering::Equal {
            return c;
        }
        t1.id.cmp(&t2.id)
    }

    fn compare_object_type_order(&self, t1: &Type, t2: &Type) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        if let (TypeData::InstantiationExpression(e1), TypeData::InstantiationExpression(e2)) =
            (&t1.data, &t2.data)
        {
            let decl1 = t1.symbol.as_ref().and_then(|s| s.declarations.first());
            let decl2 = t2.symbol.as_ref().and_then(|s| s.declarations.first());
            let c = self.compare_nodes(decl1, decl2);
            if c != Ordering::Equal {
                return c;
            }
            let c = self.compare_nodes(e1.node.as_ref(), e2.node.as_ref());
            if c != Ordering::Equal {
                return c;
            }
            return Ordering::Equal;
        }
        let c = self.compare_symbols_worker(t1.symbol.as_ref(), t2.symbol.as_ref());
        if c != Ordering::Equal {
            return c;
        }
        let (TypeData::Object(o1), TypeData::Object(o2)) = (&t1.data, &t2.data) else {
            return Ordering::Equal;
        };
        let ref1 = t1.object_flags.contains(ObjectFlags::Reference);
        let ref2 = t2.object_flags.contains(ObjectFlags::Reference);
        if ref1 && ref2 {
            let tuple1 = o1
                .target
                .as_ref()
                .is_some_and(|t| t.object_flags.contains(ObjectFlags::Tuple));
            let tuple2 = o2
                .target
                .as_ref()
                .is_some_and(|t| t.object_flags.contains(ObjectFlags::Tuple));
            if tuple1 && tuple2 {
                if let (Some(t1t), Some(t2t)) = (o1.target.as_ref(), o2.target.as_ref()) {
                    if let (TypeData::Tuple(d1), TypeData::Tuple(d2)) = (&t1t.data, &t2t.data) {
                        let c = crate::checker::mig::m3a_2::compare_tuple_types(d1, d2);
                        if c != Ordering::Equal {
                            return c;
                        }
                    }
                }
            }
            if o1.node.is_none() && o2.node.is_none() {
                return self.compare_type_lists_ordered(&o1.type_arguments, &o2.type_arguments);
            }
            let c = self.compare_nodes(o1.node.as_ref(), o2.node.as_ref());
            if c != Ordering::Equal {
                return c;
            }
            return crate::checker::mig::m3a_2::compare_type_mappers(
                o1.mapper.as_ref(),
                o2.mapper.as_ref(),
            );
        }
        if ref1 {
            return Ordering::Less;
        }
        if ref2 {
            return Ordering::Greater;
        }
        let kind_mask = OBJECT_FLAGS_CLASS_OR_INTERFACE
            .union(ObjectFlags::Reference)
            .union(ObjectFlags::Tuple)
            .union(ObjectFlags::Anonymous)
            .union(ObjectFlags::Mapped)
            .union(ObjectFlags::ReverseMapped)
            .union(ObjectFlags::EvolvingArray)
            .union(ObjectFlags::InstantiationExpressionType)
            .union(ObjectFlags::SingleSignatureType);
        let c = (t1.object_flags & kind_mask)
            .bits()
            .cmp(&(t2.object_flags & kind_mask).bits());
        if c != Ordering::Equal {
            return c;
        }
        crate::checker::mig::m3a_2::compare_type_mappers(o1.mapper.as_ref(), o2.mapper.as_ref())
    }

    fn compare_type_lists_ordered(
        &self,
        s1: &[Arc<Type>],
        s2: &[Arc<Type>],
    ) -> std::cmp::Ordering {
        if s1.len() != s2.len() {
            return s1.len().cmp(&s2.len());
        }
        for (a, b) in s1.iter().zip(s2.iter()) {
            let c = self.compare_types_ordered(a, b);
            if c != std::cmp::Ordering::Equal {
                return c;
            }
        }
        std::cmp::Ordering::Equal
    }
}
