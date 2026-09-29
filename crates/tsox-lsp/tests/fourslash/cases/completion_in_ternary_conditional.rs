use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_in_ternary_conditional() {
    let content = r#"export enum Bar { }
export enum Foo { }


function foo(x: Foo) { return x; }
function bar(z: string, x: Foo) { return x; }

const a = '';

foo(/*1*/);
bar(a, a == '' ? /*2*/);
bar(a, a == '' ? /*3*/ : /*4*/);"#;
    let mut s = Session::new_for_test("completionInTernaryConditional", content);
    // TODO: // Test marker 1 - should have Foo preselected in simple call
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: // Test marker 2 - should have Foo preselected after ? in incomplete ternary
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    // TODO: // Test marker 3 - should have Foo preselected after ? in ternary with colon
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    // TODO: // Test marker 4 - should have Foo preselected after : in ternary
    fourslash::go_to_marker(&mut s, "4");
    // TODO: f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
}
