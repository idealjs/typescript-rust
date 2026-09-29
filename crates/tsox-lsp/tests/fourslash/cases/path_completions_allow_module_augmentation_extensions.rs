use tsox_lsp::fourslash::{self, Session};


#[test]
fn path_completions_allow_module_augmentation_extensions() {
    let content = r#"// @Filename: /project/foo.css
export const foo = 0;
// @Filename: declarations.d.ts
declare module "*.css" {}
// @Filename: /project/main.ts
import {} from ".//**/""#;
    let mut s = Session::new_for_test("pathCompletionsAllowModuleAugmentationExtensions", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["foo.css"]);
}
