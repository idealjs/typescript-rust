#![allow(unused_imports)]

use crate::checker::checker_checker_checker::Checker;
use crate::checker::mig::wc3::NodeAccessExt;
use crate::checker::types::*;
use crate::checker::utilities_has_only_expression_initialization::{
    compare_type_lists, compare_types, is_const_type_reference,
};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;
use tsox_core::core::text::TextRange;
use tsox_core::diagnostics as msg;
use tsox_frontend::ast::{
    self, get_source_file_of_node, Node, NodeData, NodeList, ScriptKind, SourceFile, Symbol,
    SyntaxKind,
};
use tsox_frontend::scanner::skip_trivia;
use tsox_tsoptions::module::{get_types_package_name, mangle_scoped_package_name};

pub struct DiagnosticDetails {
    pub message: msg::Message,
    pub args: Vec<String>,
}

#[path = "r31k1_defs.rs"]
pub mod r31k1_defs;

pub fn new_diagnostic_for_node(
    node: Option<&Arc<Node>>,
    message: msg::Message,
    args: Vec<String>,
) -> ast::Diagnostic {
    let mut file = None;
    let mut loc = TextRange::default();
    if let Some(node) = node {
        file = r31k1_defs::source_file_of_node(node);
        loc = crate::checker::relater_relation::error_range_for_node(node);
    }
    ast::Diagnostic::new(file, loc, message, args)
}

pub fn new_diagnostic_chain_for_node(
    chain: Option<&ast::Diagnostic>,
    node: Option<&Arc<Node>>,
    message: msg::Message,
    args: Vec<String>,
) -> ast::Diagnostic {
    if let Some(chain) = chain {
        let mut result =
            ast::Diagnostic::new(chain.file.clone(), chain.loc, message, args);
        result.message_chain.push(chain.clone());
        result.related_information = chain.related_information.clone();
        return result;
    }
    new_diagnostic_for_node(node, message, args)
}

pub fn find_in_map<K, V, S>(m: &HashMap<K, V, S>, predicate: impl Fn(&V) -> bool) -> V
where
    K: std::hash::Hash + Eq,
    V: Clone + Default,
{
    for value in m.values() {
        if predicate(value) {
            return value.clone();
        }
    }
    V::default()
}

pub fn is_const_type_reference_name(node: &Arc<Node>) -> bool {
    ast::is_identifier(node)
        && node
            .parent()
            .is_some_and(|p| is_const_type_reference(&p) && p.parent().is_some_and(|gp| ast::is_assertion_expression(&gp)))
}

pub fn has_dot_dot_dot_token(node: &Arc<Node>) -> bool {
    let dot_dot_dot_token = match node.kind {
        SyntaxKind::Parameter => Some(&node.as_parameter_declaration().dot_dot_dot_token),
        SyntaxKind::BindingElement => match &node.data {
            NodeData::BindingElement(d) => Some(&d.dot_dot_dot_token),
            _ => None,
        },
        SyntaxKind::NamedTupleMember => match &node.data {
            NodeData::NamedTupleMember(d) => Some(&d.dot_dot_dot_token),
            _ => None,
        },
        SyntaxKind::JsxExpression => match &node.data {
            NodeData::JsxExpression(d) => Some(&d.dot_dot_dot_token),
            _ => None,
        },
        _ => None,
    };
    dot_dot_dot_token.is_some_and(|t| t.is_some())
}

pub fn compare_element_labels(n1: Option<&Arc<Node>>, n2: Option<&Arc<Node>>) -> Ordering {
    match (n1, n2) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(a), Some(b)) => a
            .name()
            .and_then(|na| b.name().map(|nb| na.text().cmp(nb.text())))
            .unwrap_or(Ordering::Equal),
    }
}

pub fn compare_tuple_types(t1: &TupleTypeData, t2: &TupleTypeData) -> Ordering {
    if t1.readonly != t2.readonly {
        return if t1.readonly { Ordering::Greater } else { Ordering::Less };
    }
    if t1.element_infos.len() != t2.element_infos.len() {
        return t1.element_infos.len().cmp(&t2.element_infos.len());
    }
    for i in 0..t1.element_infos.len() {
        let c = (t1.element_infos[i].flags.bits() as i32).cmp(&(t2.element_infos[i].flags.bits() as i32));
        if c != Ordering::Equal {
            return c;
        }
    }
    for i in 0..t1.element_infos.len() {
        let c = compare_element_labels(
            t1.element_infos[i].labeled_declaration.as_ref(),
            t2.element_infos[i].labeled_declaration.as_ref(),
        );
        if c != Ordering::Equal {
            return c;
        }
    }
    Ordering::Equal
}

pub fn compare_type_mappers(m1: Option<&Arc<TypeMapper>>, m2: Option<&Arc<TypeMapper>>) -> Ordering {
    let (Some(m1), Some(m2)) = (m1, m2) else {
        return match (m1, m2) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(_), Some(_)) => Ordering::Equal,
        };
    };
    if Arc::ptr_eq(m1, m2) {
        return Ordering::Equal;
    }
    (m1.kind as i32).cmp(&(m2.kind as i32))
}

pub fn get_containing_class_excluding_class_decorators(node: &Arc<Node>) -> Option<Arc<Node>> {
    let mut decorator = None;
    let mut current = node.parent();
    while let Some(n) = current {
        if ast::is_class_like(&n) {
            break;
        }
        if ast::is_decorator(&n) {
            decorator = Some(n);
            break;
        }
        current = n.parent();
    }
    if let Some(decorator) = decorator {
        if let Some(dp) = decorator.parent() {
            if ast::is_class_like(&dp) {
                return ast::get_containing_class(&dp);
            }
        }
        return ast::get_containing_class(&decorator);
    }
    ast::get_containing_class(node)
}

pub fn is_in_name_of_expression_with_type_arguments_or_heritage_type_reference(node: &Arc<Node>) -> bool {
    let mut node = Arc::clone(node);
    while let Some(parent) = node.parent() {
        if parent.kind != SyntaxKind::PropertyAccessExpression && parent.kind != SyntaxKind::QualifiedName {
            break;
        }
        node = parent;
    }
    node.parent().is_some_and(|parent| {
        parent.kind == SyntaxKind::ExpressionWithTypeArguments
            || tsox_frontend::ast::mig::m3g::is_name_of_heritage_clause_type_reference(&node)
    })
}

pub fn min_and_max<T>(slice: &[T], get_value: impl Fn(&T) -> i64) -> (i64, i64) {
    let mut min_value = 0;
    let mut max_value = 0;
    for (i, element) in slice.iter().enumerate() {
        let value = get_value(element);
        if i == 0 {
            min_value = value;
            max_value = value;
        } else {
            min_value = min_value.min(value);
            max_value = max_value.max(value);
        }
    }
    (min_value, max_value)
}

pub fn range_of_type_parameters(source_file: &Arc<SourceFile>, type_parameters: &NodeList) -> TextRange {
    let text: &str = &source_file.text;
    TextRange::new(
        type_parameters.pos() - 1,
        text.len().min(skip_trivia(text, type_parameters.end()) + 1),
    )
}

impl Checker {
    pub fn compare_symbols_worker(&self, s1: Option<&Arc<Symbol>>, s2: Option<&Arc<Symbol>>) -> Ordering {
        let (Some(s1), Some(s2)) = (s1, s2) else {
            return match (s1, s2) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(_), Some(_)) => Ordering::Equal,
            };
        };
        if Arc::ptr_eq(s1, s2) {
            return Ordering::Equal;
        }
        if !s1.declarations.is_empty() && !s2.declarations.is_empty() {
            let r = self.compare_nodes(Some(&s1.declarations[0]), Some(&s2.declarations[0]));
            if r != Ordering::Equal {
                return r;
            }
        } else if !s1.declarations.is_empty() {
            return Ordering::Less;
        } else if !s2.declarations.is_empty() {
            return Ordering::Greater;
        }
        let r = s1.name.cmp(&s2.name);
        if r != Ordering::Equal {
            return r;
        }
        ast::get_symbol_id(s1).cmp(&ast::get_symbol_id(s2))
    }

    pub fn compare_nodes(&self, n1: Option<&Arc<Node>>, n2: Option<&Arc<Node>>) -> Ordering {
        let (Some(n1), Some(n2)) = (n1, n2) else {
            return match (n1, n2) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(_), Some(_)) => Ordering::Equal,
            };
        };
        if Arc::ptr_eq(n1, n2) {
            return Ordering::Equal;
        }
        let s1 = get_source_file_of_node(n1);
        let s2 = get_source_file_of_node(n2);
        let same_file = match (&s1, &s2) {
            (Some(f1), Some(f2)) => Arc::ptr_eq(f1, f2),
            (None, None) => true,
            _ => false,
        };
        if !same_file {
            let f1 = s1
                .as_ref()
                .and_then(|f| self.file_index_map.get(&f.id()).copied())
                .unwrap_or(0);
            let f2 = s2
                .as_ref()
                .and_then(|f| self.file_index_map.get(&f.id()).copied())
                .unwrap_or(0);
            return f1.cmp(&f2);
        }
        n1.pos().cmp(&n2.pos())
    }

    pub fn is_mutable_local_variable_declaration(&self, declaration: &Arc<Node>) -> bool {
        let Some(parent) = declaration.parent() else {
            return false;
        };
        parent.flags.contains(tsox_frontend::ast::NodeFlags::Let)
            && !(ast::get_combined_modifier_flags(declaration).contains(ast::ModifierFlags::Export)
                || (parent
                    .parent()
                    .is_some_and(|gp| gp.kind == SyntaxKind::VariableStatement
                        && gp.parent().is_some_and(|gpp| Checker::is_global_source_file(&gpp)))))
    }

    pub fn call_like_expression_may_have_type_arguments(&self, node: &Arc<Node>) -> bool {
        tsox_frontend::ast::mig::m3f_4::is_call_or_new_expression(node)
            || ast::is_tagged_template_expression(node)
            || tsox_frontend::ast::mig::m3g::is_jsx_opening_like_element(node)
    }

    pub fn is_canceled(&self) -> bool {
        self.was_canceled()
    }

    pub fn check_not_canceled(&self) {
        if self.was_canceled() {
            panic!("Checker was previously cancelled");
        }
    }

    pub fn get_packages_map(&mut self) -> &HashMap<String, bool> {
        if self.packages_map.is_none() {
            let mut packages_map = HashMap::new();
            let resolved_modules = self.program.get_resolved_modules();
            for resolved_modules_in_file in resolved_modules.values() {
                for module in resolved_modules_in_file.iter().filter_map(|(_, module)| module.as_ref()) {
                    let Some(package_id) = module.package_id.as_ref() else {
                        continue;
                    };
                    if !package_id.name.is_empty() {
                        let name = package_id.name.clone();
                        let has_types = packages_map.get(&name).copied().unwrap_or(false)
                            || module.extension == tsox_core::tspath::EXTENSION_DTS;
                        packages_map.insert(name, has_types);
                    }
                }
            }
            self.packages_map = Some(packages_map);
        }
        self.packages_map.as_ref().unwrap()
    }

    pub fn types_package_exists(&mut self, package_name: &str) -> bool {
        let packages_map = self.get_packages_map();
        packages_map.contains_key(&get_types_package_name(package_name))
    }

    pub fn package_bundles_types(&mut self, package_name: &str) -> bool {
        let packages_map = self.get_packages_map();
        packages_map.get(package_name).copied().unwrap_or(false)
    }

    pub fn is_unchecked_js_suggestion(
        &self,
        node: Option<&Arc<Node>>,
        suggestion: Option<&Arc<Symbol>>,
        exclude_classes: bool,
    ) -> bool {
        let Some(node) = node else {
            return false;
        };
        let Some(file) = self.get_source_file_of_node(node) else {
            return false;
        };
        if self.compiler_options.check_js.is_unknown()
            && (file.script_kind == ScriptKind::Js || file.script_kind == ScriptKind::Jsx)
        {
            let mut declaration_file = None;
            if let Some(suggestion) = suggestion {
                if let Some(first) = suggestion.declarations.first() {
                    declaration_file = self.get_source_file_of_node(first);
                }
            }
            let suggestion_has_no_extends_or_decorators = suggestion.is_none()
                || suggestion.and_then(|s| s.value_declaration.as_ref()).is_none()
                || !suggestion
                    .and_then(|s| s.value_declaration.as_ref())
                    .is_some_and(|vd| ast::is_class_like(vd))
                || suggestion
                    .and_then(|s| s.value_declaration.as_ref())
                    .map(|vd| ast::get_extends_heritage_clause_elements(vd))
                    .is_none_or(|e| e.is_empty())
                || suggestion
                    .and_then(|s| s.value_declaration.as_ref())
                    .is_some_and(|vd| crate::checker::mig::wc1b::class_or_constructor_parameter_is_decorated(false, vd));
            let declaration_is_global = declaration_file
                .as_ref()
                .is_some_and(|df| Checker::is_global_source_file(&df.node));
            return !(declaration_file.is_some() && !Arc::ptr_eq(&file, declaration_file.as_ref().unwrap()) && declaration_is_global)
                && !(exclude_classes
                    && suggestion.is_some_and(|s| s.flags.contains(ast::SymbolFlags::Class))
                    && suggestion_has_no_extends_or_decorators)
                && !(exclude_classes
                    && ast::is_property_access_expression(node)
                    && node.expression().is_some_and(|e| e.kind == SyntaxKind::ThisKeyword)
                    && suggestion_has_no_extends_or_decorators);
        }
        false
    }
}
