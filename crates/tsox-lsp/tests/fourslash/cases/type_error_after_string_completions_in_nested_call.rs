use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyNonSuggestionDiagnostics"]
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
    let mut s = Session::new_for_test("typeErrorAfterStringCompletionsInNestedCall", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "x");
    fourslash::verify_completions_exact_at(&mut s, None, &["ALOHA", "ALOHAx", "LUNCH_TIME", "MORNING"]);
    fourslash::unsupported("VerifyNonSuggestionDiagnostics"); // f.VerifyNonSuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
