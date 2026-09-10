use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.ReplaceLine(t, 3, '	z: number;') // `y: number;`"]
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
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
    // TODO: f.ReplaceLine(t, 3, "	z: number;") // `y: number;`
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}
