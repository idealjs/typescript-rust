use super::*;

pub(crate) fn parse_jsdoc(source: &str) -> Arc<Node> {
    let mut parser = super::super::Parser::new(source.to_string());

    let text = source;
    let start = text.find("/**").expect("no /** found");

    let end = text[start..].find("*/").expect("no */ found") + start + 2;
    parser
        .parse_jsdoc_comment(start, end, start)
        .expect("parse failed")
}

#[test]
pub(crate) fn parse_empty_jsdoc() {
    let node = parse_jsdoc("/** */");
    assert_eq!(node.kind, SyntaxKind::JSDoc);
}

#[test]
pub(crate) fn parse_simple_comment() {
    let node = parse_jsdoc("/** This is a comment */");
    assert_eq!(node.kind, SyntaxKind::JSDoc);
    if let NodeData::JSDoc(d) = &node.data {
        assert!(!d.comment.nodes.is_empty(), "should have comment text");
        assert!(d.tags.is_none(), "should have no tags");
    } else {
        panic!("not a JSDoc node");
    }
}

#[test]
pub(crate) fn parse_param_tag() {
    let node = parse_jsdoc("/** @param {string} name The name */");
    assert_eq!(node.kind, SyntaxKind::JSDoc);
    if let NodeData::JSDoc(d) = &node.data {
        let tags = d.tags.as_ref().expect("should have tags");
        assert_eq!(tags.nodes.len(), 1);
        assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocParameterTag);
    }
}

#[test]
pub(crate) fn parse_returns_tag() {
    let node = parse_jsdoc("/** @returns {number} The result */");
    if let NodeData::JSDoc(d) = &node.data {
        let tags = d.tags.as_ref().expect("should have tags");
        assert_eq!(tags.nodes.len(), 1);
        assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocReturnTag);
    }
}

#[test]
pub(crate) fn parse_type_tag() {
    let node = parse_jsdoc("/** @type {string} */");
    if let NodeData::JSDoc(d) = &node.data {
        let tags = d.tags.as_ref().expect("should have tags");
        assert_eq!(tags.nodes.len(), 1);
        assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocTypeTag);
    }
}

#[test]
pub(crate) fn parse_deprecated_tag() {
    let node = parse_jsdoc("/** @deprecated Use newThing instead */");
    if let NodeData::JSDoc(d) = &node.data {
        let tags = d.tags.as_ref().expect("should have tags");
        assert_eq!(tags.nodes.len(), 1);
        assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocDeprecatedTag);
    }
}

#[test]
pub(crate) fn parse_multiple_tags() {
    let node = parse_jsdoc(
        "/**\n * @param {string} x First\n * @param {number} y Second\n * @returns {boolean}\n */",
    );
    if let NodeData::JSDoc(d) = &node.data {
        let tags = d.tags.as_ref().expect("should have tags");
        assert_eq!(tags.nodes.len(), 3);
        assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocParameterTag);
        assert_eq!(tags.nodes[1].kind, SyntaxKind::JSDocParameterTag);
        assert_eq!(tags.nodes[2].kind, SyntaxKind::JSDocReturnTag);
    }
}

#[test]
pub(crate) fn parse_template_tag() {
    let node = parse_jsdoc("/** @template T */");
    if let NodeData::JSDoc(d) = &node.data {
        let tags = d.tags.as_ref().expect("should have tags");
        assert_eq!(tags.nodes.len(), 1);
        assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocTemplateTag);
    }
}

#[test]
pub(crate) fn parse_typedef_tag() {
    let node = parse_jsdoc("/** @typedef {Object} MyType */");
    if let NodeData::JSDoc(d) = &node.data {
        let tags = d.tags.as_ref().expect("should have tags");
        assert_eq!(tags.nodes.len(), 1);
        assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocTypedefTag);
    }
}

#[test]
pub(crate) fn parse_callback_tag() {
    let node = parse_jsdoc("/** @callback MyCallback */");
    if let NodeData::JSDoc(d) = &node.data {
        let tags = d.tags.as_ref().expect("should have tags");
        assert_eq!(tags.nodes.len(), 1);
        assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocCallbackTag);
    }
}

#[test]
pub(crate) fn parse_see_tag() {
    let node = parse_jsdoc("/** @see OtherThing */");
    if let NodeData::JSDoc(d) = &node.data {
        let tags = d.tags.as_ref().expect("should have tags");
        assert_eq!(tags.nodes.len(), 1);
        assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocSeeTag);
    }
}

#[test]
pub(crate) fn parse_simple_tags() {
    for (tag_str, expected_kind) in [
        ("@public", SyntaxKind::JSDocPublicTag),
        ("@private", SyntaxKind::JSDocPrivateTag),
        ("@protected", SyntaxKind::JSDocProtectedTag),
        ("@readonly", SyntaxKind::JSDocReadonlyTag),
        ("@override", SyntaxKind::JSDocOverrideTag),
    ] {
        let source = format!("/** {} */", tag_str);
        let node = parse_jsdoc(&source);
        if let NodeData::JSDoc(d) = &node.data {
            let tags = d.tags.as_ref().expect("should have tags");
            assert_eq!(tags.nodes.len(), 1, "tag {} should parse", tag_str);
            assert_eq!(
                tags.nodes[0].kind, expected_kind,
                "tag {} should be {:?}",
                tag_str, expected_kind
            );
        }
    }
}

#[test]
pub(crate) fn parse_unknown_tag() {
    let node = parse_jsdoc("/** @customtag some text */");
    if let NodeData::JSDoc(d) = &node.data {
        let tags = d.tags.as_ref().expect("should have tags");
        assert_eq!(tags.nodes.len(), 1);
        assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocUnknownTag);
    }
}

#[test]
pub(crate) fn parse_throws_tag() {
    let node = parse_jsdoc("/** @throws {Error} When something goes wrong */");
    if let NodeData::JSDoc(d) = &node.data {
        let tags = d.tags.as_ref().expect("should have tags");
        assert_eq!(tags.nodes.len(), 1);
        assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocThrowsTag);
    }
}

#[test]
pub(crate) fn parse_satisfies_tag() {
    let node = parse_jsdoc("/** @satisfies {string} */");
    if let NodeData::JSDoc(d) = &node.data {
        let tags = d.tags.as_ref().expect("should have tags");
        assert_eq!(tags.nodes.len(), 1);
        assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocSatisfiesTag);
    }
}

#[test]
pub(crate) fn parse_this_tag() {
    let node = parse_jsdoc("/** @this {MyClass} */");
    if let NodeData::JSDoc(d) = &node.data {
        let tags = d.tags.as_ref().expect("should have tags");
        assert_eq!(tags.nodes.len(), 1);
        assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocThisTag);
    }
}

#[test]
pub(crate) fn parse_param_with_brackets() {
    let node = parse_jsdoc("/** @param {string} [name] Optional */");
    if let NodeData::JSDoc(d) = &node.data {
        let tags = d.tags.as_ref().expect("should have tags");
        assert_eq!(tags.nodes.len(), 1);
        let tag = &tags.nodes[0];
        assert_eq!(tag.kind, SyntaxKind::JSDocParameterTag);
        if let NodeData::JSDocParameterOrPropertyTag(td) = &tag.data {
            assert!(td.is_bracketed, "should be bracketed");
        }
    }
}

#[test]
pub(crate) fn parse_multiline_comment_with_tags() {
    let source = "/**
 * Description here.
 *
 * @param {string} name - The name
 * @param {number} age - The age
 * @returns {Person} A person object
 */";
    let node = parse_jsdoc(source);
    if let NodeData::JSDoc(d) = &node.data {
        let tags = d.tags.as_ref().expect("should have tags");
        assert_eq!(tags.nodes.len(), 3);
    }
}

#[test]
pub(crate) fn parse_link_in_comment() {
    let node = parse_jsdoc("/** See {@link Foo} for details */");
    assert_eq!(node.kind, SyntaxKind::JSDoc);

    if let NodeData::JSDoc(d) = &node.data {
        assert!(!d.comment.nodes.is_empty());
    }
}

#[test]
pub(crate) fn parse_implements_tag() {
    let node = parse_jsdoc("/** @implements {IFoo} */");
    if let NodeData::JSDoc(d) = &node.data {
        let tags = d.tags.as_ref().expect("should have tags");
        assert_eq!(tags.nodes.len(), 1);
        assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocImplementsTag);
    }
}

#[test]
pub(crate) fn parse_augments_tag() {
    let node = parse_jsdoc("/** @augments {Base} */");
    if let NodeData::JSDoc(d) = &node.data {
        let tags = d.tags.as_ref().expect("should have tags");
        assert_eq!(tags.nodes.len(), 1);
        assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocAugmentsTag);
    }
}

#[test]
pub(crate) fn parse_overload_tag() {
    let node = parse_jsdoc("/** @overload */");
    if let NodeData::JSDoc(d) = &node.data {
        let tags = d.tags.as_ref().expect("should have tags");
        assert_eq!(tags.nodes.len(), 1);
        assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocOverloadTag);
    }
}

#[test]
pub(crate) fn parse_template_with_constraint() {
    let node = parse_jsdoc("/** @template {string} T */");
    if let NodeData::JSDoc(d) = &node.data {
        let tags = d.tags.as_ref().expect("should have tags");
        assert_eq!(tags.nodes.len(), 1);
        assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocTemplateTag);
    }
}

#[test]
pub(crate) fn parse_template_multiple() {
    let node = parse_jsdoc("/** @template T,U,V */");
    if let NodeData::JSDoc(d) = &node.data {
        let tags = d.tags.as_ref().expect("should have tags");
        assert_eq!(tags.nodes.len(), 1);
        assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocTemplateTag);
        if let NodeData::JSDocTemplateTag(td) = &tags.nodes[0].data {
            assert_eq!(td.type_parameters.nodes.len(), 3);
        }
    }
}

#[test]
pub(crate) fn parse_param_name_first() {
    let node = parse_jsdoc("/** @param name {string} */");
    if let NodeData::JSDoc(d) = &node.data {
        let tags = d.tags.as_ref().expect("should have tags");
        assert_eq!(tags.nodes.len(), 1);
        assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocParameterTag);
        if let NodeData::JSDocParameterOrPropertyTag(td) = &tags.nodes[0].data {
            assert!(td.is_name_first, "should be name first");
            assert!(td.type_expression.is_some(), "should have type");
        }
    }
}

#[test]
pub(crate) fn parse_jsdoc_like_text_detection() {
    assert!(is_jsdoc_like_text("/** comment */"));
    assert!(!is_jsdoc_like_text("/**/"));
    assert!(!is_jsdoc_like_text("/* not jsdoc */"));
}

#[test]
pub(crate) fn parse_remove_trailing_whitespace() {
    let comments = vec!["hello".to_string(), "  ".to_string(), "\n".to_string()];
    let result = remove_trailing_whitespace(comments);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], "hello");
}

#[test]
pub(crate) fn parse_remove_leading_newlines() {
    let comments = vec!["\n".to_string(), "\r\n".to_string(), "hello".to_string()];
    let result = remove_leading_newlines(comments);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], "hello");
}

pub(crate) fn parse_source(source: &str) -> crate::ast::SourceFile {
    super::super::Parser::parse_source_file_text("test.ts", source.to_string())
}

pub(crate) fn first_statement(file: &crate::ast::SourceFile) -> Arc<Node> {
    use crate::ast::node_data_generated::*;
    match &file.node.data {
        NodeData::SourceFile(d) => d.statements.nodes[0].clone(),
        _ => panic!("expected SourceFile"),
    }
}

#[test]
pub(crate) fn get_jsdoc_comment_ranges_finds_leading_jsdoc() {
    let text = "/** Hello */\nconst x = 1;";
    let file = parse_source(text);
    let stmt = first_statement(&file);
    let ranges = get_jsdoc_comment_ranges(&file.text, &stmt);
    assert_eq!(ranges.len(), 1);
    assert!(text[ranges[0].pos..ranges[0].end].starts_with("/**"));
}

#[test]
pub(crate) fn get_jsdoc_comment_ranges_skips_non_jsdoc_comments() {
    let text = "/* not jsdoc */\nconst x = 1;";
    let file = parse_source(text);
    let stmt = first_statement(&file);
    let ranges = get_jsdoc_comment_ranges(&file.text, &stmt);
    assert_eq!(ranges.len(), 0, "plain /* */ comments are not JSDoc");
}

#[test]
pub(crate) fn get_jsdoc_comment_ranges_skips_empty_jsdoc() {
    let text = "/**/\nconst x = 1;";
    let file = parse_source(text);
    let stmt = first_statement(&file);
    let ranges = get_jsdoc_comment_ranges(&file.text, &stmt);
    assert_eq!(ranges.len(), 0, "/**/ is not JSDoc");
}

#[test]
pub(crate) fn parse_jsdoc_for_node_returns_parsed_tags() {
    let text = "/**\n * @param {string} name\n * @returns {void}\n */\nfunction f(name) {}\n";
    let file = parse_source(text);
    let stmt = first_statement(&file);
    let jsdocs = parse_jsdoc_for_node(&file, &stmt);
    assert_eq!(jsdocs.len(), 1);
    assert_eq!(jsdocs[0].kind, SyntaxKind::JSDoc);

    if let NodeData::JSDoc(d) = &jsdocs[0].data {
        let tags = d.tags.as_ref().expect("should have tags");
        assert_eq!(tags.nodes.len(), 2);
    }
}

#[test]
pub(crate) fn parse_jsdoc_for_node_no_comments_returns_empty() {
    let text = "const x = 1;";
    let file = parse_source(text);
    let stmt = first_statement(&file);
    let jsdocs = parse_jsdoc_for_node(&file, &stmt);
    assert!(jsdocs.is_empty());
}

#[test]
pub(crate) fn resolve_jsdoc_caches_result() {
    let text = "/** Doc */\nconst x = 1;";
    let file = parse_source(text);
    let stmt = first_statement(&file);

    let jsdocs1 = file.resolve_jsdoc(&stmt);
    assert_eq!(jsdocs1.len(), 1);

    let jsdocs2 = file.resolve_jsdoc(&stmt);
    assert_eq!(jsdocs2.len(), 1);
    assert_eq!(jsdocs1[0].kind, jsdocs2[0].kind);
}

#[test]
pub(crate) fn resolve_jsdoc_multiple_jsdoc_comments() {
    let text = "/** First */\n/** Second */\nconst x = 1;";
    let file = parse_source(text);
    let stmt = first_statement(&file);
    let jsdocs = file.resolve_jsdoc(&stmt);
    assert_eq!(jsdocs.len(), 2, "should find both JSDoc comments");
}

#[test]
pub(crate) fn node_jsdoc_returns_empty_without_flag() {
    let text = "/** Doc */\nconst x = 1;";
    let file = parse_source(text);
    let stmt = first_statement(&file);

    let jsdocs = stmt.jsdoc(&file);
    assert!(
        jsdocs.is_empty(),
        "jsdoc() should return empty without HasJSDoc flag"
    );
}
