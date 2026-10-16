use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_js_doc_tags_function_overload05() {
    let content = r#"// @Filename: quickInfoJsDocTagsFunctionOverload05.ts
declare function /*1*/foo(): void;

/**
 * @tag Tag text
 */
declare function /*2*/foo(x: number): void"#;
    let _s = Session::new_for_test("quickInfoJsDocTagsFunctionOverload05", content);
    // TODO: f.VerifyBaselineHover(t)
}
