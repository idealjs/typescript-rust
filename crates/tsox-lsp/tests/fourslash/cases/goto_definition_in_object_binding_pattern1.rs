use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn goto_definition_in_object_binding_pattern1() {
    let content = r#"function bar<T>(onfulfilled: (value: T) => void) {
  return undefined;
}
interface Test {
  /*destination*/prop2: number
}
bar<Test>(({[|pr/*goto*/op2|]})=>{});"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "goto")
}
