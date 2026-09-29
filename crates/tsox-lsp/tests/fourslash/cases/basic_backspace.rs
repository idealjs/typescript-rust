use tsox_lsp::fourslash::{self, Session};


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
    fourslash::go_to_marker(&mut s, "a");
    // TODO: f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "b");
    // TODO: f.Backspace(t, 10) // `y: number;`
    fourslash::go_to_marker(&mut s, "a");
    // TODO: f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}
