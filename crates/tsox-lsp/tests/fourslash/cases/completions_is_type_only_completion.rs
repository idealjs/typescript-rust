use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_is_type_only_completion() {
    let content = r#"// @noLib: true
// @Filename: /abc.ts
export type Abc = number;
// @Filename: /user.ts
 import { Abc } from "./abc";
function f(Abc: Ab/**/) {}"#;
    let mut s = Session::new_for_test("completionsIsTypeOnlyCompletion", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
