use tsox_lsp::fourslash::{self, Session};


#[test]
fn type_error_after_string_completions_in_nested_call2() {
    let content = r#"// @stableTypeOrdering: true
// @strict: true

type ActionFunction<
  TExpressionEvent extends { type: string },
  out TEvent extends { type: string }
> = {
  ({ event }: { event: TExpressionEvent }): void;
  _out_TEvent?: TEvent;
};

interface MachineConfig<TEvent extends { type: string }> {
  types: {
    events: TEvent;
  };
  on: {
    [K in TEvent["type"]]?: ActionFunction<
      Extract<TEvent, { type: K }>,
      TEvent
    >;
  };
}

declare function raise<
  TExpressionEvent extends { type: string },
  TEvent extends { type: string }
>(
  resolve: ({ event }: { event: TExpressionEvent }) => TEvent
): {
  ({ event }: { event: TExpressionEvent }): void;
  _out_TEvent?: TEvent;
};

declare function createMachine<TEvent extends { type: string }>(
  config: MachineConfig<TEvent>
): void;

createMachine({
  types: {
    events: {} as { type: "FOO" } | { type: "BAR" },
  },
  on: {
    [|/*error*/FOO|]: raise(({ event }) => {
      return {
        type: "BAR/*1*/" as const,
      };
    }),
  },
});"#;
    let mut s = Session::new_for_test("typeErrorAfterStringCompletionsInNestedCall2", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "x");
    fourslash::verify_completions_exact_at(&mut s, None, &["BAR", "FOO"]);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
