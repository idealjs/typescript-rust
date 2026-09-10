use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: // Test marker 1 - should have Foo preselected in simple cal"]
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
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: // Test marker 2 - should have Foo preselected after ? in incomplete ternary
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    // TODO: // Test marker 3 - should have Foo preselected after ? in ternary with colon
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    // TODO: // Test marker 4 - should have Foo preselected after : in ternary
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
}
