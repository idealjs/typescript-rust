#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn parse_error_for_missing_semicolon_after(&mut self, node: &Arc<Node>) {
        let expression_text = if node.kind == SyntaxKind::Identifier {
            node.text().to_string()
        } else {
            String::new()
        };
        if expression_text.is_empty() {
            self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &[";"]);
            return;
        }

        let pos = node.loc.pos();
        match expression_text.as_str() {
            "const" | "let" | "var" => {
                self.parse_error_at(
                    pos,
                    node.end(),
                    diagnostics::VARIABLE_DECLARATION_NOT_ALLOWED_AT_THIS_LOCATION,
                    &[],
                );
            }

            "declare" => {}
            "interface" => {
                self.parse_error_for_invalid_name(
                    diagnostics::INTERFACE_NAME_CANNOT_BE_0,
                    diagnostics::INTERFACE_MUST_BE_GIVEN_A_NAME,
                );
            }
            "is" => {
                self.parse_error_at(
                    pos,
                    self.token_pos(),
                    diagnostics::A_TYPE_PREDICATE_IS_ONLY_ALLOWED_IN_RETURN_TYPE_POSITION_FOR_FUNCTIONS_AND_METHODS,
                    &[],
                );
            }
            "module" | "namespace" => {
                self.parse_error_for_invalid_name(
                    diagnostics::NAMESPACE_NAME_CANNOT_BE_0,
                    diagnostics::NAMESPACE_MUST_BE_GIVEN_A_NAME,
                );
            }
            "type" => {
                self.parse_error_for_invalid_name(
                    diagnostics::TYPE_ALIAS_NAME_CANNOT_BE_0,
                    diagnostics::TYPE_ALIAS_MUST_BE_GIVEN_A_NAME,
                );
            }
            _ => {
                if self.token == SyntaxKind::Unknown {
                    return;
                }

                let expression_text = if node.kind == SyntaxKind::Identifier {
                    node.text().to_string()
                } else {
                    String::new()
                };
                let followed_by_identifier = {
                    let text = self.scanner.text();
                    let mut i = node.end();
                    let bytes = text.as_bytes();
                    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
                        i += 1;
                    }
                    i < bytes.len()
                        && (bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' || bytes[i] == b'$')
                };
                if !expression_text.is_empty() && followed_by_identifier {
                    let lower = expression_text.to_ascii_lowercase();
                    let mut best: Option<(usize, String)> = None;
                    for kw in KEYWORD_SUGGESTIONS {
                        if kw.len() <= 2 {
                            continue;
                        }
                        let d = crate::checker::edit_distance(&lower, &kw.to_ascii_lowercase());
                        let budget = (expression_text.len() as f64 * 0.4).floor() + 0.9;
                        if d as f64 > budget {
                            continue;
                        }
                        if best.as_ref().is_none_or(|(bd, _)| d < *bd) {
                            best = Some((d, kw.to_string()));
                        }
                    }

                    let space_sugg = best
                        .is_none()
                        .then(|| {
                            KEYWORD_SUGGESTIONS
                                .iter()
                                .find(|kw| {
                                    kw.len() > 2
                                        && expression_text.len() > kw.len() + 2
                                        && expression_text.starts_with(*kw)
                                })
                                .map(|kw| format!("{kw} {}", &expression_text[kw.len()..]))
                        })
                        .flatten();
                    if let Some((_, sugg)) = best {
                        self.parse_error_at(
                            pos,
                            node.end(),
                            diagnostics::UNKNOWN_KEYWORD_OR_IDENTIFIER_DID_YOU_MEAN_0,
                            &[&sugg],
                        );
                        return;
                    }
                    if let Some(sugg) = space_sugg {
                        self.parse_error_at(
                            pos,
                            node.end(),
                            diagnostics::UNKNOWN_KEYWORD_OR_IDENTIFIER_DID_YOU_MEAN_0,
                            &[&sugg],
                        );
                        return;
                    }
                }
                self.parse_error_at(
                    pos,
                    node.end(),
                    diagnostics::UNEXPECTED_KEYWORD_OR_IDENTIFIER,
                    &[],
                );
            }
        }
    }

    pub(crate) fn parse_error_for_invalid_name(
        &mut self,
        name_diagnostic: Message,
        blank_diagnostic: Message,
    ) {
        if self.token == SyntaxKind::OpenBraceToken {
            self.parse_error_at_current_token(blank_diagnostic, &[]);
        } else {
            let arg = self.scanner.token_text().to_string();
            self.parse_error_at_current_token(name_diagnostic, &[&arg]);
        }
    }

    pub(crate) fn parse_accessor_declaration(
        &mut self,
        pos: usize,
        modifiers: Option<Arc<ModifierList>>,
        accessor_kind: SyntaxKind,
    ) -> Arc<Node> {
        self.next_token();
        let name = self.parse_property_name();
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();
        let body = if self.token == SyntaxKind::OpenBraceToken {
            Some(self.parse_block())
        } else {
            self.parse_semicolon();
            None
        };

        let end = body
            .as_ref()
            .map_or(self.scanner.full_start_pos(), |b| b.end());
        let range = TextRange::new(pos, end);
        match accessor_kind {
            SyntaxKind::GetKeyword => Arc::new(Node::with_loc(
                SyntaxKind::GetAccessor,
                NodeData::GetAccessorDeclaration(GetAccessorDeclarationData {
                    modifiers,
                    name,
                    type_parameters,
                    parameters,
                    type_node,
                    full_signature: None,
                    body,
                }),
                range,
            )),
            _ => Arc::new(Node::with_loc(
                SyntaxKind::SetAccessor,
                NodeData::SetAccessorDeclaration(SetAccessorDeclarationData {
                    modifiers,
                    name,
                    type_parameters,
                    parameters,
                    type_node,
                    full_signature: None,
                    body,
                }),
                range,
            )),
        }
    }

    pub fn diagnostics(&self) -> &[ParserDiagnostic] {
        &self.diagnostics
    }
}
