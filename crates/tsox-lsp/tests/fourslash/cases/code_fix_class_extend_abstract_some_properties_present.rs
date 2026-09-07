use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_class_extend_abstract_some_properties_present() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: false
// @noImplicitOverride: true
abstract class A {
   abstract x: number;
   abstract y: number;
   abstract z: number;
}

class C extends A {[|   
   |]constructor(public x: number) { super(); }
   y: number;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `
}
