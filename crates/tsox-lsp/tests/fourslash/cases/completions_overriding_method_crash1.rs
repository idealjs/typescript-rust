use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_overriding_method_crash1() {
    let content = r#"// @newline: LF
// @Filename: a.ts
declare class Component<T> {
    setState(stateHandler: ((oldState: T, newState: T) => void)): void;
}

class SubComponent extends Component<{}> {
    /*$*/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "$", &fourslash.CompletionsExpectedList{
}
