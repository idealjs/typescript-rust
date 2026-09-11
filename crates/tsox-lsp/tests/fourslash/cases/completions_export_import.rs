use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_export_import() {
    let content = r#"// @lib: es5
declare global {
    namespace N {
        const foo: number;
    }
}
export import foo = N.foo;
/**/"#;
    let mut s = Session::new_for_test("completionsExportImport", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
