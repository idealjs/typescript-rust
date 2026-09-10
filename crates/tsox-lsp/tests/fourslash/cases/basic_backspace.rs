use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.Backspace(t, 10) // `y: number;`"]
#[test]
fn basic_backspace() {
    let content = r#"export {};
interface Point {
	x: number;
	y: number;/*b*/
}
declare const p: Point;
p./*a*/"#;
    let mut s = Session::new_for_test("basicBackspace", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "b");
    // TODO: f.Backspace(t, 10) // `y: number;`
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}
