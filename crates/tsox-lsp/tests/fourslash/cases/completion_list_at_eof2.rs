use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.GoToEOF"]
#[test]
fn completion_list_at_eof2() {
    let content = r#"namespace Shapes {
    export class Point {
        constructor(public x: number, public y: number) { }
    }
}
var p = <Shapes."#;
    let mut s = Session::new(content);
    fourslash::unsupported("GoToEOF"); // f.GoToEOF(t)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
