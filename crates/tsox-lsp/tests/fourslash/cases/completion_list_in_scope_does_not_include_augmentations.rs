use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_scope_does_not_include_augmentations() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: /a.ts
import * as self from "./a";

declare module "a" {
    export const a: number;
}

/**/"#;
    let mut s = Session::new_for_test("completionListInScope_doesNotIncludeAugmentations", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
