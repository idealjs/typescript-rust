use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_string_literal13() {
    let content = r#"// @lib: es5
interface SymbolConstructor {
    readonly species: symbol;
}
var Symbol: SymbolConstructor;
interface PromiseConstructor {
  [Symbol.species]: PromiseConstructor;
}
var Promise: PromiseConstructor;
Promise["/*1*/"];"#;
    let mut s = Session::new_for_test("completionForStringLiteral13", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
