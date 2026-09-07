use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn completions03() {
    let content = r#"// @lib: es5
interface Foo {
   one: any;
   two: any;
   three: any;
}

let x: Foo = {
    get one() { return "" },
    set two(t) {},
    /**/
}"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
