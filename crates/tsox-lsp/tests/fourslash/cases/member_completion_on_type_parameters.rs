use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn member_completion_on_type_parameters() {
    let content = r#"interface IFoo {
    x: number;
    y: string;
}

function foo<S, T extends IFoo, U extends Object, V extends IFoo>() {
    var s:S, t: T, u: U, v: V;
    s./*S*/;    // no constraint, no completion
    t./*T*/;    // IFoo
    u./*U*/;    // IFoo
    v./*V*/;    // IFoo
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "S", nil)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"T", "V"}, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "U", &fourslash.CompletionsExpectedList{
}
