use tsox_lsp::fourslash::Session;


#[test]
fn arity_error_after_string_completions() {
    let content = r#"// @strict: true

interface Events {
  click: any;
  drag: any;
}

declare function addListener<K extends keyof Events>(type: K, listener: (ev: Events[K]) => any): void;

/*1*/addListener/*2*/("/*3*/")"#;
    let _s = Session::new_for_test("arityErrorAfterStringCompletions", content);
    // TODO: f.VerifyCompletions(t, []string{"3"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyErrorExistsBetweenMarkers(t, "1", "2")
}
