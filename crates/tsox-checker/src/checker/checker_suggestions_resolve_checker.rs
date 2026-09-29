#![allow(unused_imports)]

use crate::checker::checker_suggestions_resolve::*;

impl Checker {
    pub(crate) fn find_name_suggestion(&self, name: &str, meaning: SymbolFlags) -> Option<String> {
        let mut candidates: Vec<&Arc<Symbol>> = Vec::new();
        let symbol_map = self.program.symbol_map();
        fn push_symbol<'a>(
            cands: &mut Vec<&'a Arc<Symbol>>,
            sym: &'a Arc<Symbol>,
            meaning: SymbolFlags,
        ) {
            if sym.flags.intersects(meaning) {
                cands.push(sym);
            }
        }

        if let Some(file) = self.current_file.as_ref() {
            let fid = file.id();
            if let Some(locals) = symbol_map.locals.get(&fid) {
                for sym in locals.entries.values() {
                    push_symbol(&mut candidates, sym, meaning);
                }
            }

            if let Some(sym) = symbol_map.symbols.get(&fid) {
                for sub in sym.members.entries.values() {
                    push_symbol(&mut candidates, sub, meaning);
                }
                for sub in sym.exports.entries.values() {
                    push_symbol(&mut candidates, sub, meaning);
                }
            }
        }
        for &container_id in self.scope_stack.iter() {
            if let Some(locals) = symbol_map.locals.get(&container_id) {
                for sym in locals.entries.values() {
                    push_symbol(&mut candidates, sym, meaning);
                }
            }
            if let Some(sym) = symbol_map.symbols.get(&container_id) {
                for sub in sym.members.entries.values() {
                    push_symbol(&mut candidates, sub, meaning);
                }
                for sub in sym.exports.entries.values() {
                    push_symbol(&mut candidates, sub, meaning);
                }
            }
        }
        for sym in self.globals.entries.values() {
            push_symbol(&mut candidates, sym, meaning);
        }

        let rune_len = name.chars().count();
        let maximum_length_difference = ((rune_len as f64) * 0.34) as usize;
        let maximum_length_difference = maximum_length_difference.max(2);
        let mut best_distance = ((rune_len as f64) * 0.4).floor() + 0.9;
        let mut best: Option<((usize, usize), &String)> = None;
        for sym in candidates {
            let cand: &String = &sym.name;

            if cand.is_empty()
                || cand.starts_with('"')
                || cand.starts_with('\'')
                || cand.starts_with('`')
                || cand.starts_with('\u{FE}')
            {
                continue;
            }
            let cand_len = cand.chars().count();
            if cand_len < 3 && !cand.eq_ignore_ascii_case(name) {
                continue;
            }
            if rune_len.max(cand_len) - rune_len.min(cand_len) > maximum_length_difference {
                continue;
            }
            if cand == name {
                continue;
            }
            let Some(d) = levenshtein_with_max(name, cand, best_distance) else {
                continue;
            };

            let key = self.suggestion_order_key(sym);
            let replace = match &best {
                None => true,
                Some((bkey, _)) => {
                    if d < best_distance {
                        true
                    } else {
                        key < *bkey
                    }
                }
            };
            if d < best_distance {
                best_distance = d;
            }
            if replace {
                best = Some((key, cand));
            }
        }
        best.map(|(_, c)| c.clone())
    }

    pub(crate) fn suggestion_order_key(&self, sym: &Arc<Symbol>) -> (usize, usize) {
        let Some(decl) = sym.declarations.first() else {
            return (usize::MAX, usize::MAX);
        };
        let Some(sf) = self.get_source_file_of_node(decl) else {
            return (usize::MAX, usize::MAX);
        };
        let idx = self
            .files
            .iter()
            .position(|f| f.node.id() == sf.node.id())
            .unwrap_or(usize::MAX);
        (idx, decl.loc.pos())
    }

    pub(crate) fn inside_function_body(node: &Arc<Node>) -> bool {
        let mut anc = node.parent();
        while let Some(a) = anc {
            match a.kind {
                SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::Constructor
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor => return true,
                SyntaxKind::ModuleBlock
                | SyntaxKind::SourceFile
                | SyntaxKind::ModuleDeclaration => return false,
                _ => {}
            }
            anc = a.parent();
        }
        false
    }

    pub(crate) fn check_class_heritage_members(&mut self, node: &Arc<Node>) {
        let (name, type_parameters, members, is_class_expression) = match &node.data {
            tsox_frontend::ast::NodeData::ClassDeclaration(d) => {
                (&d.name, d.type_parameters.as_ref(), &d.members, false)
            }
            tsox_frontend::ast::NodeData::ClassExpression(d) => {
                (&d.name, d.type_parameters.as_ref(), &d.members, true)
            }
            _ => return,
        };
        let Some((base_node, _base_sym)) = self.extends_base_of(node) else {
            return;
        };
        let class_name = name
            .as_ref()
            .map(|n| n.text().to_string())
            .unwrap_or_default();
        let class_name = self.generic_display_name(&class_name, type_parameters);
        let base_name = self
            .extends_heritage_expr_of(node)
            .and_then(|e| self.node_source_text(&e))
            .unwrap_or_else(|| Self::class_name_text(&base_node));

        if !node.has_syntactic_modifier(ModifierFlags::Abstract) {
            let mut missing: Vec<String> = Vec::new();
            Self::collect_unimplemented_abstract_members(node, &base_node, &mut missing);
            missing.dedup();
            if !missing.is_empty() {
                let file = self.current_file.clone();
                if is_class_expression {
                    let quoted = missing
                        .iter()
                        .map(|m| format!("'{m}'"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let (message, args) = if missing.len() == 1 {
                        (
                            tsox_core::diagnostics::messages_generated::
                                NON_ABSTRACT_CLASS_EXPRESSION_DOES_NOT_IMPLEMENT_INHERITED_ABSTRACT_MEMBER_0_FROM_CLASS_1,
                            vec![missing[0].clone(), base_name.clone()],
                        )
                    } else if missing.len() > 5 {
                        let first4 = missing[..4]
                            .iter()
                            .map(|m| format!("'{m}'"))
                            .collect::<Vec<_>>()
                            .join(", ");
                        (
                            tsox_core::diagnostics::messages_generated::
                                NON_ABSTRACT_CLASS_EXPRESSION_IS_MISSING_IMPLEMENTATIONS_FOR_THE_FOLLOWING_MEMBERS_OF_0_COLON_1_AND_2_MORE,
                            vec![
                                base_name.clone(),
                                first4,
                                (missing.len() - 4).to_string(),
                            ],
                        )
                    } else {
                        (
                            tsox_core::diagnostics::messages_generated::
                                NON_ABSTRACT_CLASS_EXPRESSION_IS_MISSING_IMPLEMENTATIONS_FOR_THE_FOLLOWING_MEMBERS_OF_0_COLON_1,
                            vec![base_name.clone(), quoted],
                        )
                    };
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        file, node.loc, message, args,
                    ));
                } else {
                    let name_loc = name.as_ref().map(|n| n.loc).unwrap_or(node.loc);
                    if missing.len() == 1 {
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            name_loc,
                            tsox_core::diagnostics::messages_generated::
                                NON_ABSTRACT_CLASS_0_DOES_NOT_IMPLEMENT_INHERITED_ABSTRACT_MEMBER_1_FROM_CLASS_2,
                            vec![class_name.clone(), missing[0].clone(), base_name.clone()],
                        ));
                    } else {
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            name_loc,
                            tsox_core::diagnostics::messages_generated::
                                NON_ABSTRACT_CLASS_0_IS_MISSING_IMPLEMENTATIONS_FOR_THE_FOLLOWING_MEMBERS_OF_1_COLON_2,
                            vec![
                                class_name.clone(),
                                base_name.clone(),
                                missing
                                    .iter()
                                    .map(|m| format!("'{m}'"))
                                    .collect::<Vec<_>>()
                                    .join(", "),
                            ],
                        ));
                    }
                }
            }
        }

        for member in members.iter() {
            // 候选成员名/类型：直接属性成员 + 构造器参数属性
            //（constructor(public xyz: number)，带可访问性修饰的形参即类成员）
            let mut candidates: Vec<(&Arc<Node>, Option<Arc<Type>>)> = Vec::new();
            match &member.data {
                tsox_frontend::ast::NodeData::PropertyDeclaration(pd) => {
                    if pd.name.kind == SyntaxKind::Identifier {
                        let t = if let Some(tn) = &pd.type_node {
                            Some(self.get_type_from_type_node(tn))
                        } else {
                            pd.initializer
                                .as_ref()
                                .map(|init| self.get_type_of_node(init))
                        };
                        candidates.push((&pd.name, t));
                    }
                }
                tsox_frontend::ast::NodeData::GetAccessorDeclaration(gd) => {
                    if gd.name.kind == SyntaxKind::Identifier {
                        let t = if let Some(tn) = &gd.type_node {
                            Some(self.get_type_from_type_node(tn))
                        } else {
                            Self::first_return_expression(gd.body.as_ref())
                                .map(|e| self.get_type_of_node(&e))
                        };
                        candidates.push((&gd.name, t));
                    }
                }
                tsox_frontend::ast::NodeData::ConstructorDeclaration(cd) => {
                    for p in cd.parameters.iter() {
                        if let tsox_frontend::ast::NodeData::ParameterDeclaration(pdd) = &p.data
                            && pdd.name.kind == SyntaxKind::Identifier
                            && pdd
                                .modifiers
                                .as_ref()
                                .is_some_and(|m| !m.list.nodes.is_empty())
                        {
                            let t = pdd
                                .type_node
                                .as_ref()
                                .map(|tn| self.get_type_from_type_node(tn));
                            candidates.push((&pdd.name, t));
                        }
                    }
                }
                _ => {}
            }
            for (name_node, own_type) in candidates {
                self.check_member_override_compatibility(
                    name_node,
                    own_type,
                    &base_node,
                    &class_name,
                    &base_name,
                );
            }
        }
    }

    fn check_member_override_compatibility(
        &mut self,
        name_node: &Arc<Node>,
        own_type: Option<Arc<Type>>,
        base_node: &Arc<Node>,
        _class_name: &str,
        _base_name: &str,
    ) {
        let Some(own_type) = own_type else { return };
        {
            let prop_name = name_node.text().to_string();
            let Some(base_member) = Self::find_class_member_by_name(&base_node, &prop_name) else {
                return;
            };
            let base_tn = match &base_member.data {
                tsox_frontend::ast::NodeData::PropertyDeclaration(pd) => pd.type_node.clone(),
                tsox_frontend::ast::NodeData::GetAccessorDeclaration(gd) => gd.type_node.clone(),
                tsox_frontend::ast::NodeData::SetAccessorDeclaration(sd) => {
                    sd.parameters.iter().next().and_then(|p| {
                        if let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) = &p.data {
                            pd.type_node.clone()
                        } else {
                            None
                        }
                    })
                }
                _ => None,
            };
            let Some(base_tn) = base_tn else {
                return;
            };
            let base_type = self.get_type_from_type_node(&base_tn);
            // TS2416 由 check_heritage_clause 的 Go 对齐实现（含错误链）发射，
            // 此遗留路径不再重复报
            if !own_type.flags.contains(TypeFlags::Any)
                && !self.is_type_assignable_to(&own_type, &base_type)
            {
            }
        }
    }

    pub(crate) fn class_members_of(class: &Arc<Node>) -> &Arc<NodeList> {
        match &class.data {
            tsox_frontend::ast::NodeData::ClassDeclaration(d) => &d.members,
            tsox_frontend::ast::NodeData::ClassExpression(d) => &d.members,
            _ => {
                static EMPTY: std::sync::OnceLock<Arc<NodeList>> = std::sync::OnceLock::new();
                EMPTY.get_or_init(|| Arc::new(NodeList::default()))
            }
        }
    }

    pub(crate) fn find_class_member_by_name(class: &Arc<Node>, name: &str) -> Option<Arc<Node>> {
        Self::class_members_of(class)
            .iter()
            .find(|m| {
                let n = match &m.data {
                    tsox_frontend::ast::NodeData::PropertyDeclaration(d) => &d.name,
                    tsox_frontend::ast::NodeData::MethodDeclaration(d) => &d.name,
                    tsox_frontend::ast::NodeData::GetAccessorDeclaration(d) => &d.name,
                    tsox_frontend::ast::NodeData::SetAccessorDeclaration(d) => &d.name,
                    _ => return false,
                };
                n.kind == SyntaxKind::Identifier && n.text() == name
            })
            .cloned()
    }
}
