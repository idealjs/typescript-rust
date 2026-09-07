use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn type_error_after_string_completions_in_nested_call() {
    let content = r#"// @stableTypeOrdering: true
// @strict: true

type GreetingEvent =
  | { type: "MORNING" }
  | { type: "LUNCH_TIME" }
  | { type: "ALOHA" };

interface RaiseActionObject<TEvent extends { type: string }> {
  type: "raise";
  event: TEvent;
}

declare function raise<TEvent extends { type: string }>(
  ev: TEvent
): RaiseActionObject<TEvent>;

declare function createMachine<TEvent extends { type: string }>(config: {
  actions: RaiseActionObject<TEvent>;
}): void;

createMachine<GreetingEvent>({
  [|/*error*/actions|]: raise({ type: "ALOHA/*1*/" }),
});"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "x");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyNonSuggestionDiagnostics"); // f.VerifyNonSuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
