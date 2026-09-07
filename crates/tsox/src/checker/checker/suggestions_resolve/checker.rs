#![allow(unused_imports)]

use super::*;

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
        let mut anc = node.parent.as_ref();
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
            anc = a.parent.as_ref();
        }
        false
    }

    pub(crate) fn check_class_heritage_members(&mut self, node: &Arc<Node>) {
        let crate::ast::NodeData::ClassDeclaration(data) = &node.data else {
            return;
        };
        let Some((base_node, _base_sym)) = self.extends_base_of(node) else {
            return;
        };
        let class_name = data
            .name
            .as_ref()
            .map(|n| n.text().to_string())
            .unwrap_or_default();
        let base_name = Self::class_name_text(&base_node);

        if !node.has_syntactic_modifier(ModifierFlags::Abstract) {
            let mut missing: Vec<String> = Vec::new();
            Self::collect_unimplemented_abstract_members(node, &base_node, &mut missing);
            missing.dedup();
            if !missing.is_empty() {
                let file = self.current_file.clone();
                let name_loc = data.name.as_ref().map(|n| n.loc).unwrap_or(node.loc);
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    name_loc,
                    crate::diagnostics::messages_generated::
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

        for member in data.members.iter() {
            let (name_node, own_type): (&Arc<Node>, Option<Arc<Type>>) = match &member.data {
                crate::ast::NodeData::PropertyDeclaration(pd) => {
                    if pd.name.kind != SyntaxKind::Identifier {
                        continue;
                    }
                    let t = if let Some(tn) = &pd.type_node {
                        Some(self.get_type_from_type_node(tn))
                    } else {
                        pd.initializer
                            .as_ref()
                            .map(|init| self.get_type_of_node(init))
                    };
                    (&pd.name, t)
                }
                crate::ast::NodeData::GetAccessorDeclaration(gd) => {
                    if gd.name.kind != SyntaxKind::Identifier {
                        continue;
                    }

                    let t = if let Some(tn) = &gd.type_node {
                        Some(self.get_type_from_type_node(tn))
                    } else {
                        Self::first_return_expression(gd.body.as_ref())
                            .map(|e| self.get_type_of_node(&e))
                    };
                    (&gd.name, t)
                }
                _ => continue,
            };
            let Some(own_type) = own_type else { continue };
            let prop_name = name_node.text().to_string();
            let Some(base_member) = Self::find_class_member_by_name(&base_node, &prop_name) else {
                continue;
            };
            let base_tn = match &base_member.data {
                crate::ast::NodeData::PropertyDeclaration(pd) => pd.type_node.clone(),
                crate::ast::NodeData::GetAccessorDeclaration(gd) => gd.type_node.clone(),
                crate::ast::NodeData::SetAccessorDeclaration(sd) => {
                    sd.parameters.iter().next().and_then(|p| {
                        if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data {
                            pd.type_node.clone()
                        } else {
                            None
                        }
                    })
                }
                _ => None,
            };
            let Some(base_tn) = base_tn else {
                continue;
            };
            let base_type = self.get_type_from_type_node(&base_tn);
            if !own_type.flags.contains(TypeFlags::Any)
                && !self.is_type_assignable_to(&own_type, &base_type)
            {
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    name_node.loc,
                    crate::diagnostics::messages_generated::
                        PROPERTY_0_IN_TYPE_1_IS_NOT_ASSIGNABLE_TO_THE_SAME_PROPERTY_IN_BASE_TYPE_2,
                    vec![
                        prop_name,
                        class_name.clone(),
                        base_name.clone(),
                    ],
                ));
            }
        }
    }

    pub(crate) fn class_members_of(class: &Arc<Node>) -> &Arc<NodeList> {
        match &class.data {
            crate::ast::NodeData::ClassDeclaration(d) => &d.members,
            crate::ast::NodeData::ClassExpression(d) => &d.members,
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
                    crate::ast::NodeData::PropertyDeclaration(d) => &d.name,
                    crate::ast::NodeData::MethodDeclaration(d) => &d.name,
                    crate::ast::NodeData::GetAccessorDeclaration(d) => &d.name,
                    crate::ast::NodeData::SetAccessorDeclaration(d) => &d.name,
                    _ => return false,
                };
                n.kind == SyntaxKind::Identifier && n.text() == name
            })
            .cloned()
    }
}
