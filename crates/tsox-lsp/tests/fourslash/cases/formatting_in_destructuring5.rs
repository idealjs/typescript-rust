use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn formatting_in_destructuring5() {
    let content = r#"let a, b;
/*1*/if (false)[a, b] = [1, 2];
/*2*/if (true)        [a, b] = [1, 2];
/*3*/var a = [1, 2, 3].map(num => num) [0];"#;
    let mut s = Session::new_for_test("formattingInDestructuring5", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"if (false) [a, b] = [1, 2];"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"if (true) [a, b] = [1, 2];"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"var a = [1, 2, 3].map(num => num)[0];"#);
}
