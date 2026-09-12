use tsox_lsp::fourslash::{self, Session};


#[test]
fn remove_exported_class_from_reopened_module() {
    let content = r#"namespace multiM { }

namespace multiM {
    /*1*/export class c { }
}
"#;
    let mut s = Session::new_for_test("removeExportedClassFromReopenedModule", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.DeleteAtCaret(t, 18)
    fourslash::go_to_eof(&mut s, );
    fourslash::insert(&mut s, "new multiM.c();");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
