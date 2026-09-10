use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
#[test]
fn completions_overriding_method22() {
    let content = r#"// @newline: LF
// @Filename: b.ts
export type C = { x: number }
export interface ExtShape {
    $returnPromise(id: number): Promise<C>;
    $return(id: number): C;
}
// @Filename: test.ts
import { ExtShape } from './b';
abstract class ExtBase implements ExtShape {
    public [|$/**/|]
}"#;
    let mut s = Session::new_for_test("completionsOverridingMethod22", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
