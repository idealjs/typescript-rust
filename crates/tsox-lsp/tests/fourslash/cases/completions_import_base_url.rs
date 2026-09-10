use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
