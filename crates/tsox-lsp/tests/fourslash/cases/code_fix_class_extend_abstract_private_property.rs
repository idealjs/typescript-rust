use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_class_extend_abstract_private_property() {
    let content = r#"// @noImplicitOverride: true
abstract class A {
   private abstract x: number;
   m() { this.x; } // Avoid unused private
}

class C extends A {[| |]}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}
