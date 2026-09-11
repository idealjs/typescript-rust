use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_on_class_merged_with_function() {
    let content = r#"namespace Test {
    class Mocked {
        myProp: string;
    }
    class Tester {
        willThrowError() {
            Mocked = Mocked || function () { // => Error: Invalid left-hand side of assignment expression.
                return { /**/myProp: "test" };
            };
        }
    }
}"#;
    let mut s = Session::new_for_test("quickInfoOnClassMergedWithFunction", content);
    fourslash::verify_quick_info_at(&mut s, "", "(property) myProp: string", "");
}
