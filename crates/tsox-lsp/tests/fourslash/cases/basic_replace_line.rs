use tsox_lsp::fourslash::{self, Session};


#[test]
fn basic_replace_line() {
    let content = r#"export {};
interface Point {
	x: number;
	y: number;
}
declare const p: Point;
p./*a*/"#;
    let mut s = Session::new_for_test("basicReplaceLine", content);
    fourslash::go_to_marker(&mut s, "a");
    // TODO: f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
    // TODO: f.ReplaceLine(t, 3, "	z: number;") // `y: number;`
    fourslash::go_to_marker(&mut s, "a");
    // TODO: f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}
