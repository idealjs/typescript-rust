use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_type_formatting() {
    let content = r#"var y: import("./c2").mytype;
var z: import ("./c2").mytype;"#;
    let mut s = Session::new_for_test("importTypeFormatting", content);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"var y: import("./c2").mytype;
var z: import("./c2").mytype;"#);
}
