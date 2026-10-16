use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_after_class_extends() {
    let content = r#"namespace Bar {
    export class Bleah {
    }
    export class Foo extends /**/Bleah {
    }
}

function Blah(x: Bar.Bleah) {
}"#;
    let mut s = Session::new_for_test("completionListAfterClassExtends", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["Bar", "Bleah", "Foo"], &[]);
}
