use tsox_lsp::fourslash::{self, Session};


#[test]
fn inverted_clodule_after_quick_info() {
    let content = r#"// @strict: false
namespace M {
    namespace A {
        var o;
    }
    class A {
        /**/c
    }
}"#;
    let mut s = Session::new_for_test("invertedCloduleAfterQuickInfo", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyQuickInfoExists(t)
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
