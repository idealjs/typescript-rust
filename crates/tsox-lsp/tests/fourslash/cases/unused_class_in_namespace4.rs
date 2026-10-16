use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_class_in_namespace4() {
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
    let _s = Session::new_for_test("unusedClassInNamespace4", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `namespace Validation {
}
