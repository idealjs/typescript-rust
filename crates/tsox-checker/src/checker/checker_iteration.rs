#![allow(unused_imports)]

use crate::checker::checker::*;

#[derive(Default, Clone)]
pub(crate) struct IterationTypes {
    pub yield_type: Option<Arc<Type>>,
    pub return_type: Option<Arc<Type>>,
    pub next_type: Option<Arc<Type>>,
}

impl IterationTypes {
    pub(crate) fn has_types(&self) -> bool {
        self.yield_type.is_some() || self.return_type.is_some() || self.next_type.is_some()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum IterationUse {
    ForOf { for_await: bool },
    Spread,
    Destructuring,
    Element,
    YieldStar { is_async: bool },
    GeneratorReturnType { is_async: bool },
}

impl IterationUse {
    fn allows_async_iterables(self) -> bool {
        matches!(
            self,
            IterationUse::ForOf { for_await: true }
                | IterationUse::YieldStar { is_async: true }
                | IterationUse::GeneratorReturnType { is_async: true }
        )
    }

    fn allows_string_input(self) -> bool {
        matches!(self, IterationUse::ForOf { .. })
    }
}

const SYNC_ITERATION_GLOBALS: &[&str] =
    &["Iterable", "Iterator", "IterableIterator", "Generator"];
const ASYNC_ITERATION_GLOBALS: &[&str] =
    &["AsyncIterable", "AsyncIterator", "AsyncIterableIterator", "AsyncGenerator"];
const BUILTIN_ITERATOR_GLOBALS: &[&str] =
    &["ArrayIterator", "MapIterator", "SetIterator", "StringIterator"];

impl Checker {
    pub(crate) fn check_iterated_type_or_element_type(
        &mut self,
        use_: IterationUse,
        input: &Arc<Type>,
        error_node: Option<&Arc<Node>>,
    ) -> Arc<Type> {
        if input.flags.contains(TypeFlags::Any) {
            return Arc::clone(input);
        }
        self.get_iterated_type_or_element_type(use_, input, error_node)
            .unwrap_or_else(|| self.get_any_type())
    }

    pub(crate) fn get_iterated_type_or_element_type(
        &mut self,
        use_: IterationUse,
        input: &Arc<Type>,
        error_node: Option<&Arc<Node>>,
    ) -> Option<Arc<Type>> {
        if input.flags.contains(TypeFlags::Never) {
            self.report_type_not_iterable_error(error_node, input, use_.allows_async_iterables());
            return None;
        }
        let iterable_exists = self.global_type_declares_members("Iterable");
        if iterable_exists || use_.allows_async_iterables() {
            let types = self.iteration_types_of_iterable(use_, input, error_node);
            if let Some(y) = types.yield_type {
                return Some(y);
            }
            if iterable_exists {
                return None;
            }
        }
        self.get_array_like_element_type(use_, input, error_node)
    }

    fn get_array_like_element_type(
        &mut self,
        use_: IterationUse,
        input: &Arc<Type>,
        error_node: Option<&Arc<Node>>,
    ) -> Option<Arc<Type>> {
        let mut array_type = Arc::clone(input);
        let mut has_string_constituent = false;
        if use_.allows_string_input() {
            if let Some(constituents) = input.types() {
                let non_string: Vec<Arc<Type>> = constituents
                    .iter()
                    .filter(|c| !c.flags.intersects(TypeFlags::String | TypeFlags::StringLiteral))
                    .cloned()
                    .collect();
                if non_string.len() != constituents.len() {
                    has_string_constituent = true;
                    array_type = if non_string.is_empty() {
                        self.never_type()
                    } else {
                        self.get_union_type(non_string)
                    };
                }
            } else if input
                .flags
                .intersects(TypeFlags::String | TypeFlags::StringLiteral)
            {
                has_string_constituent = true;
                array_type = self.never_type();
            }
        }
        if has_string_constituent && array_type.flags.contains(TypeFlags::Never) {
            return Some(self.string_type());
        }
        if !self.is_array_like_type(&array_type) {
            if error_node.is_some() {
                let type_str = self.type_to_string(&array_type);
                let message = if has_string_constituent {
                    tsox_core::diagnostics::messages_generated::TYPE_0_IS_NOT_AN_ARRAY_TYPE_OR_A_STRING_TYPE
                } else {
                    tsox_core::diagnostics::messages_generated::TYPE_0_IS_NOT_AN_ARRAY_TYPE
                };
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    error_node.unwrap().loc,
                    message,
                    vec![type_str],
                ));
            }
            if has_string_constituent {
                return Some(self.string_type());
            }
            return None;
        }
        let element = self.get_number_index_type(&array_type);
        if has_string_constituent && let Some(element) = &element {
            let string_t = self.string_type();
            if element.flags.intersects(TypeFlags::String | TypeFlags::StringLiteral) {
                return Some(string_t);
            }
            return Some(self.get_union_type(vec![Arc::clone(element), string_t]));
        }
        element
    }

    pub(crate) fn report_type_not_iterable_error(
        &mut self,
        error_node: Option<&Arc<Node>>,
        t: &Arc<Type>,
        allow_async: bool,
    ) {
        let Some(node) = error_node else { return };
        let type_str = self.type_to_string(t);
        let message = if allow_async {
            tsox_core::diagnostics::messages_generated::
                TYPE_0_MUST_HAVE_A_SYMBOL_ASYNCITERATOR_METHOD_THAT_RETURNS_AN_ASYNC_ITERATOR
        } else {
            tsox_core::diagnostics::messages_generated::
                TYPE_0_MUST_HAVE_A_SYMBOL_ITERATOR_METHOD_THAT_RETURNS_AN_ITERATOR
        };
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            self.current_file.clone(),
            node.loc,
            message,
            vec![type_str],
        ));
    }

    pub(crate) fn iteration_types_of_iterable(
        &mut self,
        use_: IterationUse,
        t: &Arc<Type>,
        error_node: Option<&Arc<Node>>,
    ) -> IterationTypes {
        if t.flags.contains(TypeFlags::Any) {
            let any = self.get_any_type();
            return IterationTypes {
                yield_type: Some(Arc::clone(&any)),
                return_type: Some(Arc::clone(&any)),
                next_type: Some(any),
            };
        }
        if let Some(constituents) = t.types().filter(|_| t.is_union()) {
            let mut all: Vec<IterationTypes> = Vec::new();
            for constituent in constituents {
                let part =
                    self.iteration_types_of_iterable_worker(use_, constituent, None);
                if !part.has_types() {
                    self.report_type_not_iterable_error(error_node, t, use_.allows_async_iterables());
                    return IterationTypes::default();
                }
                all.push(part);
            }
            return self.combine_iteration_types(all);
        }
        self.iteration_types_of_iterable_worker(use_, t, error_node)
    }

    fn iteration_types_of_iterable_worker(
        &mut self,
        use_: IterationUse,
        t: &Arc<Type>,
        error_node: Option<&Arc<Node>>,
    ) -> IterationTypes {
        if t
            .flags
            .intersects(TypeFlags::String | TypeFlags::StringLiteral)
        {
            let string_t = self.string_type();
            let any = self.get_any_type();
            return IterationTypes {
                yield_type: Some(string_t),
                return_type: Some(any),
                next_type: Some(self.unknown_type()),
            };
        }
        if self.is_array_type(t) {
            let element = self.get_array_element_type(t);
            let any = self.get_any_type();
            return IterationTypes {
                yield_type: Some(element),
                return_type: Some(any),
                next_type: Some(self.unknown_type()),
            };
        }
        if self.is_tuple_type(t) {
            let mut elements: Vec<Arc<Type>> = Vec::new();
            let mut index = 0;
            while let Some(element) = self.get_tuple_element_type(t, index) {
                elements.push(element);
                index += 1;
            }
            let yield_type = if elements.is_empty() {
                self.never_type()
            } else {
                self.get_union_type(elements)
            };
            let any = self.get_any_type();
            return IterationTypes {
                yield_type: Some(yield_type),
                return_type: Some(any),
                next_type: Some(self.unknown_type()),
            };
        }
        if use_.allows_async_iterables() {
            let types = self.iteration_types_from_property(
                t,
                "__@asyncIterator",
                ASYNC_ITERATION_GLOBALS,
                error_node,
                true,
            );
            if types.has_types() {
                return types;
            }
        }
        let sync_types = self.iteration_types_from_property(
            t,
            "__@iterator",
            SYNC_ITERATION_GLOBALS,
            error_node,
            false,
        );
        if sync_types.has_types() {
            return sync_types;
        }
        self.report_type_not_iterable_error(error_node, t, use_.allows_async_iterables());
        IterationTypes::default()
    }

    #[allow(clippy::too_many_arguments)]
    fn iteration_types_from_property(
        &mut self,
        t: &Arc<Type>,
        method_name: &str,
        fast_path_globals: &[&str],
        error_node: Option<&Arc<Node>>,
        is_async: bool,
    ) -> IterationTypes {
        let fast = self.iteration_types_from_reference(t, fast_path_globals);
        if fast.has_types() {
            return fast;
        }
        let builtin = self.iteration_types_from_reference(t, BUILTIN_ITERATOR_GLOBALS);
        if builtin.has_types() {
            let any = self.get_any_type();
            return IterationTypes {
                yield_type: builtin.yield_type,
                return_type: Some(any),
                next_type: Some(self.unknown_type()),
            };
        }
        let Some(method) = self.get_property_of_type(t, method_name) else {
            return IterationTypes::default();
        };
        if method.flags.contains(SymbolFlags::Optional) {
            return IterationTypes::default();
        }
        let method_type = self.get_type_of_symbol(&method);
        if method_type.flags.contains(TypeFlags::Any) {
            let any = self.get_any_type();
            return IterationTypes {
                yield_type: Some(Arc::clone(&any)),
                return_type: Some(Arc::clone(&any)),
                next_type: Some(any),
            };
        }
        let mut iterator_types: Vec<Arc<Type>> = Vec::new();
        for sig in self.get_signatures_of_type(&method_type, SignatureKind::Call) {
            if self.get_min_argument_count(&sig) == 0
                && let Some(rt) = self.get_return_type_of_signature(&sig)
            {
                iterator_types.push(rt);
            }
        }
        if iterator_types.is_empty() {
            return IterationTypes::default();
        }
        let iterator_type = if iterator_types.len() == 1 {
            iterator_types.into_iter().next().expect("exactly one")
        } else {
            self.get_intersection_type(iterator_types)
        };
        let mut pending: Vec<tsox_frontend::ast::Diagnostic> = Vec::new();
        let result = self.iteration_types_of_iterator(
            &iterator_type,
            fast_path_globals,
            error_node,
            is_async,
            &mut pending,
        );
        if result.has_types() {
            for diagnostic in pending {
                self.diagnostics.add(diagnostic);
            }
        }
        result
    }

    pub(crate) fn iteration_types_of_iterator(
        &mut self,
        t: &Arc<Type>,
        fast_path_globals: &[&str],
        error_node: Option<&Arc<Node>>,
        is_async: bool,
        pending: &mut Vec<tsox_frontend::ast::Diagnostic>,
    ) -> IterationTypes {
        if t.flags.contains(TypeFlags::Any) {
            let any = self.get_any_type();
            return IterationTypes {
                yield_type: Some(Arc::clone(&any)),
                return_type: Some(Arc::clone(&any)),
                next_type: Some(any),
            };
        }
        let fast = self.iteration_types_from_reference(t, fast_path_globals);
        if fast.has_types() {
            return fast;
        }
        let builtin = self.iteration_types_from_reference(t, BUILTIN_ITERATOR_GLOBALS);
        if builtin.has_types() {
            let any = self.get_any_type();
            return IterationTypes {
                yield_type: builtin.yield_type,
                return_type: Some(any),
                next_type: Some(self.unknown_type()),
            };
        }
        let next_types = self.iteration_types_of_method(t, "next", error_node, is_async, pending);
        let return_types =
            self.iteration_types_of_method(t, "return", error_node, is_async, pending);
        let throw_types =
            self.iteration_types_of_method(t, "throw", error_node, is_async, pending);
        self.combine_iteration_types(vec![next_types, return_types, throw_types])
    }
    fn iteration_types_of_method(
        &mut self,
        t: &Arc<Type>,
        method_name: &str,
        error_node: Option<&Arc<Node>>,
        is_async: bool,
        pending: &mut Vec<tsox_frontend::ast::Diagnostic>,
    ) -> IterationTypes {
        let method = self.get_property_of_type(t, method_name);
        if method.is_none() && method_name != "next" {
            return IterationTypes::default();
        }
        let mut method_type: Option<Arc<Type>> = None;
        if let Some(method) = &method
            && !(method_name == "next" && method.flags.contains(SymbolFlags::Optional))
        {
            let mt = self.get_type_of_symbol(method);
            method_type = Some(if method_name == "next" {
                mt
            } else {
                self.get_non_nullable_type_of(&mt)
            });
        }
        if let Some(mt) = &method_type
            && mt.flags.contains(TypeFlags::Any)
        {
            let any = self.get_any_type();
            return IterationTypes {
                yield_type: Some(Arc::clone(&any)),
                return_type: Some(Arc::clone(&any)),
                next_type: Some(any),
            };
        }
        let mut signatures: Vec<Arc<Signature>> = Vec::new();
        if let Some(mt) = &method_type {
            signatures = self.get_signatures_of_type(mt, SignatureKind::Call);
        }
        if signatures.is_empty() {
            if let Some(node) = error_node {
                let message = if method_name == "next" {
                    if is_async {
                        tsox_core::diagnostics::messages_generated::
                            AN_ASYNC_ITERATOR_MUST_HAVE_A_NEXT_METHOD
                    } else {
                        tsox_core::diagnostics::messages_generated::AN_ITERATOR_MUST_HAVE_A_NEXT_METHOD
                    }
                } else if is_async {
                    tsox_core::diagnostics::messages_generated::
                        THE_0_PROPERTY_OF_AN_ASYNC_ITERATOR_MUST_BE_A_METHOD
                } else {
                    tsox_core::diagnostics::messages_generated::
                        THE_0_PROPERTY_OF_AN_ITERATOR_MUST_BE_A_METHOD
                };
                pending.push(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    node.loc,
                    message,
                    vec![method_name.to_string()],
                ));
            }
            return IterationTypes::default();
        }
        let mut parameter_types: Vec<Arc<Type>> = Vec::new();
        let mut return_types: Vec<Arc<Type>> = Vec::new();
        for sig in &signatures {
            if method_name != "throw" && !sig.parameters.is_empty() {
                parameter_types.push(self.get_type_at_position(sig, 0));
            }
            if let Some(rt) = self.get_return_type_of_signature(sig) {
                return_types.push(rt);
            }
        }
        let mut next_type: Option<Arc<Type>> = None;
        let mut combined_returns: Vec<Arc<Type>> = Vec::new();
        if method_name != "throw" {
            let parameter_type = if parameter_types.is_empty() {
                self.unknown_type()
            } else {
                self.get_union_type(parameter_types)
            };
            match method_name {
                "next" => next_type = Some(parameter_type),
                "return" => combined_returns.push(parameter_type),
                _ => {}
            }
        }
        let method_return_type = if return_types.is_empty() {
            self.never_type()
        } else {
            self.get_intersection_type(return_types)
        };
        let result_types = self.iteration_types_of_iterator_result(&method_return_type);
        let yield_type;
        if !result_types.has_types() {
            if let Some(node) = error_node {
                pending.push(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    node.loc,
                    tsox_core::diagnostics::messages_generated::
                        THE_TYPE_RETURNED_BY_THE_0_METHOD_OF_AN_ITERATOR_MUST_HAVE_A_VALUE_PROPERTY,
                    vec![method_name.to_string()],
                ));
            }
            let any = self.get_any_type();
            yield_type = Some(Arc::clone(&any));
            combined_returns.push(any);
        } else {
            yield_type = result_types.yield_type;
            combined_returns.extend(result_types.return_type);
        }
        IterationTypes {
            yield_type,
            return_type: Some(self.get_union_type(combined_returns)),
            next_type,
        }
    }

    pub(crate) fn iteration_types_of_iterator_result(
        &mut self,
        t: &Arc<Type>,
    ) -> IterationTypes {
        if t.flags.contains(TypeFlags::Any) {
            let any = self.get_any_type();
            return IterationTypes {
                yield_type: Some(Arc::clone(&any)),
                return_type: Some(Arc::clone(&any)),
                next_type: Some(any),
            };
        }
        let yield_result = self.filter_iterator_result(t, IterationResultKind::Yield);
        let yield_type = yield_result
            .as_ref()
            .and_then(|yt| self.get_property_of_type(yt, "value"))
            .map(|v| self.get_type_of_symbol(&v));
        let return_result = self.filter_iterator_result(t, IterationResultKind::Return);
        let return_type = return_result
            .as_ref()
            .and_then(|rt| self.get_property_of_type(rt, "value"))
            .map(|v| self.get_type_of_symbol(&v));
        if yield_type.is_none() && return_type.is_none() {
            return IterationTypes::default();
        }
        IterationTypes {
            yield_type,
            return_type: Some(return_type.unwrap_or_else(|| self.void_type())),
            next_type: None,
        }
    }

    fn filter_iterator_result(
        &mut self,
        t: &Arc<Type>,
        kind: IterationResultKind,
    ) -> Option<Arc<Type>> {
        if let Some(constituents) = t.types().filter(|_| t.is_union()) {
            let kept: Vec<Arc<Type>> = constituents
                .iter()
                .filter(|c| self.is_iterator_result_kind(c, kind))
                .cloned()
                .collect();
            if kept.is_empty() {
                return None;
            }
            return Some(self.get_union_type(kept));
        }
        if self.is_iterator_result_kind(t, kind) {
            return Some(Arc::clone(t));
        }
        None
    }

    fn is_iterator_result_kind(&mut self, t: &Arc<Type>, kind: IterationResultKind) -> bool {
        let done_type = match self.get_property_of_type(t, "done") {
            Some(done) => self.get_type_of_symbol(&done),
            None => self.false_type(),
        };
        let probe = match kind {
            IterationResultKind::Yield => self.false_type(),
            IterationResultKind::Return => self.true_type(),
        };
        self.is_type_assignable_to(&probe, &done_type)
    }

    fn iteration_types_from_reference(
        &self,
        t: &Arc<Type>,
        global_names: &[&str],
    ) -> IterationTypes {
        let target_symbol = t
            .target()
            .and_then(|target| target.symbol.clone())
            .or_else(|| t.symbol.clone());
        let Some(symbol) = target_symbol else {
            return IterationTypes::default();
        };
        if !global_names.iter().any(|name| *name == symbol.name) {
            return IterationTypes::default();
        }
        let args = t.as_object().map(|o| o.type_arguments.clone()).unwrap_or_default();
        if args.is_empty() {
            return IterationTypes::default();
        }
        IterationTypes {
            yield_type: Some(Arc::clone(&args[0])),
            return_type: args.get(1).cloned(),
            next_type: args.get(2).cloned(),
        }
    }

    fn combine_iteration_types(&mut self, parts: Vec<IterationTypes>) -> IterationTypes {
        let union_of = |parts: &[IterationTypes],
                        pick: &dyn Fn(&IterationTypes) -> Option<&Arc<Type>>|
         -> Option<Vec<Arc<Type>>> {
            let types: Vec<Arc<Type>> = parts
                .iter()
                .filter_map(|p| pick(p).cloned())
                .collect();
            if types.is_empty() {
                None
            } else {
                Some(types)
            }
        };
        let yields = union_of(&parts, &|p| p.yield_type.as_ref());
        let returns = union_of(&parts, &|p| p.return_type.as_ref());
        let nexts = union_of(&parts, &|p| p.next_type.as_ref());
        IterationTypes {
            yield_type: yields.map(|ts| self.get_union_type(ts)),
            return_type: returns.map(|ts| self.get_union_type(ts)),
            next_type: nexts.map(|ts| self.get_union_type(ts)),
        }
    }

    pub(crate) fn global_type_declares_members(&self, name: &str) -> bool {
        self.globals.get(name).is_some_and(|sym| {
            sym.flags.contains(SymbolFlags::Interface)
                || sym
                    .declarations
                    .iter()
                    .any(|d| matches!(d.data, NodeData::InterfaceDeclaration(_)))
        })
    }

    pub(crate) fn check_right_hand_side_of_for_of(
        &mut self,
        statement: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        let NodeData::ForInOrOfStatement(data) = &statement.data else {
            return None;
        };
        let nullish_word = match data.expression.kind {
            SyntaxKind::NullKeyword => Some("null"),
            SyntaxKind::UndefinedKeyword => Some("undefined"),
            SyntaxKind::Identifier if data.expression.text() == "undefined" => Some("undefined"),
            _ => None,
        };
        if let Some(word) = nullish_word {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                data.expression.loc,
                tsox_core::diagnostics::messages_generated::THE_VALUE_0_CANNOT_BE_USED_HERE,
                vec![word.to_string()],
            ));
            return None;
        }
        let for_await = data.await_modifier.is_some();
        let expression_type = self.get_type_of_node(&data.expression);
        self.get_iterated_type_or_element_type(
            IterationUse::ForOf { for_await },
            &expression_type,
            Some(&data.expression),
        )
    }

    pub(crate) fn check_for_of_reference_expression(&mut self, var_expr: &Arc<Node>) {
        let mut node = Arc::clone(var_expr);
        while node.kind == SyntaxKind::ParenthesizedExpression
            && let Some(inner) = node.expression()
        {
            node = Arc::clone(inner);
        }
        if node.kind != SyntaxKind::Identifier && !tsox_frontend::ast::is_access_expression(&node) {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                var_expr.loc,
                tsox_core::diagnostics::messages_generated::
                    THE_LEFT_HAND_SIDE_OF_A_FOR_OF_STATEMENT_MUST_BE_A_VARIABLE_OR_A_PROPERTY_ACCESS,
                Vec::new(),
            ));
        }
    }
}

#[derive(Clone, Copy)]
enum IterationResultKind {
    Yield,
    Return,
}
