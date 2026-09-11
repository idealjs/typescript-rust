use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_from_ambient_module() {
    let content = r#"// @module: esnext
// @Filename: /a.ts
declare module "m" {
    export const x: number;
}
// @Filename: /b.ts
/**/"#;
    let mut s = Session::new_for_test("completionsImport_fromAmbientModule", content);
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
