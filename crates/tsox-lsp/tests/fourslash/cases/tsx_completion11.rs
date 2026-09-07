use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn tsx_completion11() {
    let content = r#"//@module: commonjs
//@jsx: preserve
//@Filename: exporter.tsx
export class Thing { }
//@Filename: file.tsx
import {Thing} from './exporter';
var x1 = <div></**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
