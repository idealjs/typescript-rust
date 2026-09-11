use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_export_import() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @lib: es5
declare global {
    namespace N {
        const foo: number;
    }
}
export import foo = N.foo;
/**/"#;
    let mut s = Session::new_for_test("completionsExportImport", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
