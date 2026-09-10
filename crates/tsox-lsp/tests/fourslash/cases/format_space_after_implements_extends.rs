use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_space_after_implements_extends() {
    let content = r#"class C1 implements Array<string>{
}

class C2 implements Number{
}

class C3 extends Array<string>{
}

class C4 extends Number{
}"#;
    let mut s = Session::new_for_test("formatSpaceAfterImplementsExtends", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"class C1 implements Array<string> {
}

class C2 implements Number {
}

class C3 extends Array<string> {
}

class C4 extends Number {
}"#);
}
