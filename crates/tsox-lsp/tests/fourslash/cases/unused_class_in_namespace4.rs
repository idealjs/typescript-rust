use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_class_in_namespace4() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: false
// @noUnusedLocals: true
// @noUnusedParameters:true
 [| namespace Validation {
    class c1 {

    }

    export class c2 {

    }

    class c3 {
        public x: c1;
    }
} |]"#;
    let mut s = Session::new_for_test("unusedClassInNamespace4", content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `namespace Validation {
}
