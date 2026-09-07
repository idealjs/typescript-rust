use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: IsIncomplete: false,"]
#[test]
fn completions01() {
    let content = r#"// @lib: es5
var x: string[] = [];
x.forEach(function (y) { y/*1*/
x.forEach(y => y/*2*/"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, ".");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    // TODO: f.Insert(t, "});")
    // TODO: IsIncomplete: false,
    // TODO: ItemDefaults: &fourslash.CompletionsExpectedItemDefaults{
    // TODO: Items: &fourslash.CompletionsExpectedItems{
    // TODO: })
}
