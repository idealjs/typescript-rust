use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn arity_error_after_string_completions() {
    let content = r#"// @strict: true

interface Events {
  click: any;
  drag: any;
}

declare function addListener<K extends keyof Events>(type: K, listener: (ev: Events[K]) => any): void;

/*1*/addListener/*2*/("/*3*/")"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"3"}, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyErrorExistsBetweenMarkers"); // f.VerifyErrorExistsBetweenMarkers(t, "1", "2")
}
