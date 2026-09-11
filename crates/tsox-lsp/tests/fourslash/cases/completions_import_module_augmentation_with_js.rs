use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_module_augmentation_with_js() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @noEmit: true
// @Filename: /test.js
class Abcde {
    x
}

module.exports = {
    Abcde
};
// @Filename: /index.ts
export {};
declare module "./test" {
    interface Abcde { b: string }
}

Abcde/**/"#;
    let mut s = Session::new_for_test("completionsImportModuleAugmentationWithJS", content);
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
