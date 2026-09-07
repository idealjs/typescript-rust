use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.DeleteAtCaret"]
#[test]
fn remove_exported_class_from_reopened_module() {
    let content = r#"namespace multiM { }

namespace multiM {
    /*1*/export class c { }
}
"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 18)
    fourslash::unsupported("GoToEOF"); // f.GoToEOF(t)
    fourslash::insert(&mut s, "new multiM.c();");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
