use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_entry_for_union_property2() {
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
    let mut s = Session::new_for_test("completionEntryForUnionProperty2", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
