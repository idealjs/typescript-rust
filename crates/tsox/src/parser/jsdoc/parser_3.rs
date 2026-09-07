#![allow(unused_imports)]

use super::*;

impl crate::parser::Parser {
    pub(crate) fn parse_tag(&mut self, margin: usize) -> Arc<Node> {
        debug_assert_eq!(self.token, SyntaxKind::AtToken);
        let start = self.token_pos();
        self.next_token_jsdoc();
        let tag_name = self.parse_jsdoc_identifier_name(Some(diagnostics::IDENTIFIER_EXPECTED));
        let indent_text = self.skip_whitespace_or_asterisk();
        let tag_text = tag_name.text().to_string();

        let tag = match tag_text.as_str() {
            "implements" => self.parse_implements_tag(start, tag_name, margin, &indent_text),
            "augments" | "extends" => {
                self.parse_augments_tag(start, tag_name, margin, &indent_text)
            }
            "public" => self.parse_simple_tag(
                start,
                tag_name,
                margin,
                &indent_text,
                SyntaxKind::JSDocPublicTag,
            ),
            "private" => self.parse_simple_tag(
                start,
                tag_name,
                margin,
                &indent_text,
                SyntaxKind::JSDocPrivateTag,
            ),
            "protected" => self.parse_simple_tag(
                start,
                tag_name,
                margin,
                &indent_text,
                SyntaxKind::JSDocProtectedTag,
            ),
            "readonly" => self.parse_simple_tag(
                start,
                tag_name,
                margin,
                &indent_text,
                SyntaxKind::JSDocReadonlyTag,
            ),
            "override" => self.parse_simple_tag(
                start,
                tag_name,
                margin,
                &indent_text,
                SyntaxKind::JSDocOverrideTag,
            ),
            "deprecated" => self.parse_deprecated_tag(start, tag_name, margin, &indent_text),
            "this" => self.parse_this_tag(start, tag_name, margin, &indent_text),
            "arg" | "argument" | "param" => self.parse_parameter_or_property_tag(
                start,
                tag_name,
                PropertyLikeParse(PropertyLikeParse::PARAMETER),
                margin,
            ),
            "return" | "returns" => self.parse_return_tag(start, tag_name, margin, &indent_text),
            "template" => self.parse_template_tag(start, tag_name, margin, &indent_text),
            "type" => self.parse_type_tag(start, tag_name, margin, &indent_text),
            "typedef" => self.parse_typedef_tag(start, tag_name, margin, &indent_text),
            "callback" => self.parse_callback_tag(start, tag_name, margin, &indent_text),
            "overload" => self.parse_overload_tag(start, tag_name, margin, &indent_text),
            "satisfies" => self.parse_satisfies_tag(start, tag_name, margin, &indent_text),
            "see" => self.parse_see_tag(start, tag_name, margin, &indent_text),
            "exception" | "throws" => self.parse_throws_tag(start, tag_name, margin, &indent_text),
            "import" => self.parse_import_tag(start, tag_name, margin, &indent_text),
            _ => self.parse_unknown_tag(start, tag_name, margin, &indent_text),
        };
        tag
    }

    pub(crate) fn parse_simple_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
        kind: SyntaxKind,
    ) -> Arc<Node> {
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment.end();
        let data = match kind {
            SyntaxKind::JSDocPublicTag => NodeData::JSDocPublicTag(JSDocPublicTagData {
                tag_name,
                comment: Some(comment),
            }),
            SyntaxKind::JSDocPrivateTag => NodeData::JSDocPrivateTag(JSDocPrivateTagData {
                tag_name,
                comment: Some(comment),
            }),
            SyntaxKind::JSDocProtectedTag => NodeData::JSDocProtectedTag(JSDocProtectedTagData {
                tag_name,
                comment: Some(comment),
            }),
            SyntaxKind::JSDocReadonlyTag => NodeData::JSDocReadonlyTag(JSDocReadonlyTagData {
                tag_name,
                comment: Some(comment),
            }),
            SyntaxKind::JSDocOverrideTag => NodeData::JSDocOverrideTag(JSDocOverrideTagData {
                tag_name,
                comment: Some(comment),
            }),
            _ => unreachable!(),
        };
        Arc::new(Node::with_loc(kind, data, TextRange::new(start, end)))
    }

    pub(crate) fn parse_deprecated_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment.end();
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocDeprecatedTag,
            NodeData::JSDocDeprecatedTag(JSDocDeprecatedTagData {
                tag_name,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    pub(crate) fn parse_unknown_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment.end();
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocUnknownTag,
            NodeData::JSDocUnknownTag(JSDocUnknownTagData {
                tag_name,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    pub(crate) fn parse_return_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let type_expression = self.try_parse_type_expression();
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment
            .end()
            .max(type_expression.as_ref().map(|t| t.end()).unwrap_or(0));
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocReturnTag,
            NodeData::JSDocReturnTag(JSDocReturnTagData {
                tag_name,
                type_expression,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    pub(crate) fn parse_type_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let type_expression = self.parse_jsdoc_type_expression(true);
        let comment = if margin != usize::MAX {
            Some(self.parse_trailing_tag_comments(
                self.token_pos(),
                self.token_end(),
                margin,
                indent_text,
            ))
        } else {
            None
        };
        let end = comment
            .as_ref()
            .map(|c| c.end())
            .max(Some(type_expression.end()))
            .unwrap_or(type_expression.end());
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocTypeTag,
            NodeData::JSDocTypeTag(JSDocTypeTagData {
                tag_name,
                type_expression,
                comment,
            }),
            TextRange::new(start, end),
        ))
    }

    pub(crate) fn parse_this_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let type_expression = self.parse_jsdoc_type_expression(true);
        self.skip_whitespace();
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment.end().max(type_expression.end());
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocThisTag,
            NodeData::JSDocThisTag(JSDocThisTagData {
                tag_name,
                type_expression,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    pub(crate) fn parse_satisfies_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let type_expression = self.parse_jsdoc_type_expression(false);
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment.end().max(type_expression.end());
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocSatisfiesTag,
            NodeData::JSDocSatisfiesTag(JSDocSatisfiesTagData {
                tag_name,
                type_expression,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }
}
