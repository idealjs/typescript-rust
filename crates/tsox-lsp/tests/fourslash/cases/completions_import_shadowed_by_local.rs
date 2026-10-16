use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_shadowed_by_local() {
    let content = r#"// @noLib: true
// @Filename: /a.ts
export const foo = 0;
// @Filename: /b.ts
const foo = 1;
fo/**/"#;
    let mut s = Session::new_for_test("completionsImport_shadowedByLocal", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
