use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_js_doc_tags_function_overload03() {
    let content = r#"// @Filename: quickInfoJsDocTagsFunctionOverload03.ts
declare function /*1*/foo(): void;

/**
 * Doc foo overloaded
 * @tag Tag text
 */
declare function /*2*/foo(x: number): void"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
