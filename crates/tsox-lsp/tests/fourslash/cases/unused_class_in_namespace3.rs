use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_class_in_namespace3() {
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
    let _s = Session::new_for_test("unusedClassInNamespace3", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `namespace Validation {
}
