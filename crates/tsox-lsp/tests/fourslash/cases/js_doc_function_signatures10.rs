use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn js_doc_function_signatures10() {
    let content = r#"// @allowJs: true
// @Filename: Foo.js
/**
 * Do some foo things
 * @template T A Foolish template
 * @param {T} x a parameter
 */
function foo(x) {
}

fo/**/o()"#;
    let mut s = Session::new_for_test("jsDocFunctionSignatures10", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyQuickInfoIs(t, "function foo<any>(x: any): void", "Do some foo things")
}
