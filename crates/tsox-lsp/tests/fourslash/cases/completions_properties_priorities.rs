use tsox_lsp::fourslash::Session;


#[test]
fn completions_properties_priorities() {
    let content = r#"// @strict: true
interface I {
  B?: number;
  a: number;
  c?: string;
  d: string
}
const foo = {
  a: 1,
  B: 2
}
const i: I = {
  ...foo,
  /*a*/
}"#;
    let _s = Session::new_for_test("completionsPropertiesPriorities", content);
    // TODO: f.VerifyCompletions(t, []string{"a"}, &fourslash.CompletionsExpectedList{
}
