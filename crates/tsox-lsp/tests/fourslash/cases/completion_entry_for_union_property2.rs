use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_entry_for_union_property2() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @lib: es5
interface One {
    commonProperty: string;
    commonFunction(): number;
    anotherProperty: Record<string, number>;
}

interface Two {
    commonProperty: number;
    commonFunction(): number;
    anotherProperty: { foo: number }
}

var x : One | Two;

x.commonProperty./*1*/;
x.anotherProperty./*2*/;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
