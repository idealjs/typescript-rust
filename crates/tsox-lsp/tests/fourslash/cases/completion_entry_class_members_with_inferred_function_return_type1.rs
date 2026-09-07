use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_entry_class_members_with_inferred_function_return_type1() {
    let content = r#"// @filename: /tokenizer.ts
export default abstract class Tokenizer {
  errorBuilder() {
    return (pos: number, lineStart: number, curLine: number) => {};
  }
}
// @filename: /expression.ts
import Tokenizer from "./tokenizer.js";

export default abstract class ExpressionParser extends Tokenizer {
  /**/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
