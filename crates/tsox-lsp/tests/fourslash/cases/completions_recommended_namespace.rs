use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_recommended_namespace() {
    let content = r#"// @noLib: true
// @Filename: /a.ts
export namespace Name {
    export class C {}
}
export function f(c: Name.C) {}
f(new N/*a0*/);
f(new /*a1*/);
// @Filename: /b.ts
import { f } from "./a";
f(new N/*b0*/);
f(new /*b1*/);
// @Filename: /c.ts
import * as alpha from "./a";
alpha.f(new a/*c0*/);
alpha.f(new /*c1*/);"#;
    let mut s = Session::new_for_test("completionsRecommended_namespace", content);
    // TODO: f.VerifyCompletions(t, []string{"a0", "a1"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"b0", "b1"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"c0", "c1"}, &fourslash.CompletionsExpectedList{
}
