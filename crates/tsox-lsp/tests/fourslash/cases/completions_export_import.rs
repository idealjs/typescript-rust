use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
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
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
