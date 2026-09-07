use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_for_self_type_parameter_in_constraint1() {
    let content = r#"type StateMachine<Config> = {
  initial?: "states" extends keyof Config ? keyof Config["states"] : never;
  states?: Record<string, {}>;
};
declare function createMachine<Config extends StateMachine</*1*/>>(
  config: Config,
): void;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
}
