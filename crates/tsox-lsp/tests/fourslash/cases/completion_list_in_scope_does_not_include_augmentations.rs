use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_list_in_scope_does_not_include_augmentations() {
    let content = r#"// @Filename: /a.ts
import * as self from "./a";

declare module "a" {
    export const a: number;
}

/**/"#;
    let mut s = Session::new_for_test("completionListInScope_doesNotIncludeAugmentations", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
