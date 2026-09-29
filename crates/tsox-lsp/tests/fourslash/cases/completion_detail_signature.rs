use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_detail_signature() {
    let content = r#"

/*a*/

function foo(x: string): string;
function foo(x: number): number;
function foo(x: any): any {
    return x;
}"#;
    let mut s = Session::new_for_test("completionDetailSignature", content);
    fourslash::go_to_marker(&mut s, "a");
    // TODO: f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}
