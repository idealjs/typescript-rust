use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_js_doc_tags_function_overload03() {
    let content = r#"// @Filename: quickInfoJsDocTagsFunctionOverload03.ts
declare function /*1*/foo(): void;

/**
 * Doc foo overloaded
 * @tag Tag text
 */
declare function /*2*/foo(x: number): void"#;
    let _s = Session::new_for_test("quickInfoJsDocTagsFunctionOverload03", content);
    // TODO: f.VerifyBaselineHover(t)
}
