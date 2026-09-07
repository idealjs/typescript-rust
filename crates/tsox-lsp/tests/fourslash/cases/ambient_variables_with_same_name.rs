use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.GoToEOF"]
#[test]
fn ambient_variables_with_same_name() {
    let content = r#"declare namespace M {
    export var x: string;
}
declare var x: number;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("GoToEOF"); // f.GoToEOF(t)
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "")
    fourslash::verify_no_errors(&mut s);
}
