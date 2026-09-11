use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_self_declaring1() {
    let content = r#"interface Test {
  keyPath?: string;
  autoIncrement?: boolean;
}

function test<T extends Record<string, Test>>(opt: T) { }

test({
  a: {
    keyPath: '',
    [|a|]/**/
  }
})"#;
    let mut s = Session::new_for_test("completionsSelfDeclaring1", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
