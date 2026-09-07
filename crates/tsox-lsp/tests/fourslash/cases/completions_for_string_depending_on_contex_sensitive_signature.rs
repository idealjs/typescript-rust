use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_for_string_depending_on_contex_sensitive_signature() {
    let content = r#"// @strict: true

type ActorRef<TEvent extends { type: string }> = {
  send: (ev: TEvent) => void
}

type Action<TContext> = {
  (ctx: TContext): void
}

type Config<TContext> = {
  entry: Action<TContext>
}

declare function createMachine<TContext>(config: Config<TContext>): void

type EventFrom<T> = T extends ActorRef<infer TEvent> ? TEvent : never

declare function sendTo<
  TContext,
  TActor extends ActorRef<any>
>(
  actor: ((ctx: TContext) => TActor),
  event: EventFrom<TActor>
): Action<TContext>

createMachine<{
  child: ActorRef<{ type: "EVENT" }>;
}>({
  entry: sendTo((ctx) => ctx.child, { type: /*1*/ }),
});

createMachine<{
  child: ActorRef<{ type: "EVENT" }>;
}>({
  entry: sendTo((ctx) => ctx.child, { type: "/*2*/" }),
});"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
