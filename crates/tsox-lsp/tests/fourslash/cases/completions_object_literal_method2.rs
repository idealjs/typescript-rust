use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_object_literal_method2() {
    let content = r#"// @newline: LF
// @Filename: a.ts
export interface IFoo {
    bar(x: number): void;
}
// @Filename: b.ts
import { IFoo } from "./a";
export interface IBar {
    foo(f: IFoo): void;
}
// @Filename: c.ts
import { IBar } from "./b";
const obj: IBar = {
    /*a*/
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}
