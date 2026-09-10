use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn auto_import_verbatim_type_only1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @module: node18
// @verbatimModuleSyntax: true
// @Filename: /mod.ts
export const value = 0;
export class C { constructor(v: any) {} }
export interface I {}
// @Filename: /a.mts
const x: /**/"#;
    let mut s = Session::new_for_test("autoImportVerbatimTypeOnly1", content);
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
    fourslash::insert(&mut s, "I = new C");
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, nil, &fourslash.ApplyCodeActionFromCompletionOptions{
}
