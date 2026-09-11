use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_for_self_type_parameter_in_constraint1() {
    let content = r#"type StateMachine<Config> = {
  initial?: "states" extends keyof Config ? keyof Config["states"] : never;
  states?: Record<string, {}>;
};
declare function createMachine<Config extends StateMachine</*1*/>>(
  config: Config,
): void;"#;
    let mut s = Session::new_for_test("completionsForSelfTypeParameterInConstraint1", content);
    // TODO: f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
}
