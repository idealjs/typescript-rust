use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNumberOfErrorsInCurrentFile"]
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
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyQuickInfoExists"); // f.VerifyQuickInfoExists(t)
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 1)
}
