use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixAvailable"]
#[test]
fn code_fix_class_implement_interface_with_negative_number() {
    let content = r#"interface X { value: -1 | 0 | 1; }
class Y implements X { }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFixAvailable"); // f.VerifyCodeFixAvailable(t, []string{"Implement interface 'X'"})
}
