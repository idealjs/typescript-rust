use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_class_in_namespace3() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noUnusedLocals: true
// @noUnusedParameters:true
 [| namespace Validation {
    class c1 {

    }

    export class c2 {

    }

    class c3 extends c1 {

    }
} |]"#;
    let mut s = Session::new_for_test("unusedClassInNamespace3", content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `namespace Validation {
}
