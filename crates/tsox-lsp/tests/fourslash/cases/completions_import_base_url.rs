use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_base_url() {
    let content = r#"// @Filename: /tsconfig.json
{
    "compilerOptions": {
        "baseUrl": ".",
        "module": "esnext"
    }
}
// @Filename: /src/a.ts
export const foo = 0;
// @Filename: /src/b.ts
fo/**/"#;
    let mut s = Session::new_for_test("completionsImportBaseUrl", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
