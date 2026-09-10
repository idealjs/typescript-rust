use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_remove_space_between_dot_dot_dot_and_type_name() {
    let content = r#"let a: [... any[]];
let b: [...   number[]];
let c: [...     string[]];"#;
    let mut s = Session::new_for_test("formatRemoveSpaceBetweenDotDotDotAndTypeName", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"let a: [...any[]];
let b: [...number[]];
let c: [...string[]];"#);
}
