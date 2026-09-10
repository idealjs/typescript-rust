use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_cladule() {
    let content = r#"class Foo {
    doStuff(): number { return 0; }
    static staticMethod() {}
}
namespace Foo {
    export var x: number;
}
Foo/*c1*/; // should get "x", "prototype"
var s: Foo/*c2*/; // no types, in Foo, so shouldnt have anything
var f = new Foo();
f/*c3*/;"#;
    let mut s = Session::new_for_test("completionListCladule", content);
    fourslash::go_to_marker(&mut s, "c1");
    fourslash::insert(&mut s, ".");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "c2");
    fourslash::insert(&mut s, ".");
    fourslash::verify_completions_empty_at(&mut s, None);
    fourslash::go_to_marker(&mut s, "c3");
    fourslash::insert(&mut s, ".");
    fourslash::verify_completions_exact_at(&mut s, None, &["doStuff"]);
}
