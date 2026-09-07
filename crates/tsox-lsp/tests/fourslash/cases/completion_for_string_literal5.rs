use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_for_string_literal5() {
    let content = r#"// @stableTypeOrdering: true
interface Foo {
    foo: string;
    bar: string;
}

function f<K extends keyof Foo>(a: K) { };
f("/*1*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
