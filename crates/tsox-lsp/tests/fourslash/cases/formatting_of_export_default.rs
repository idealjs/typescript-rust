use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn formatting_of_export_default() {
    let content = r#"namespace Foo {
/*1*/    export        default        class        Test { }
}
/*2*/export        default        function        bar() { }"#;
    let mut s = Session::new_for_test("formattingOfExportDefault", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"    export default class Test { }"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"export default function bar() { }"#);
}
