use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn auto_import_verbatim_type_only1() {
    let content = r#"// @module: node18
// @verbatimModuleSyntax: true
// @Filename: /mod.ts
export const value = 0;
export class C { constructor(v: any) {} }
export interface I {}
// @Filename: /a.mts
const x: /**/"#;
    let mut s = Session::new_for_test("autoImportVerbatimTypeOnly1", content);
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
    fourslash::insert(&mut s, "I = new C");
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, nil, &fourslash.ApplyCodeActionFromCompletionOptions{
}
