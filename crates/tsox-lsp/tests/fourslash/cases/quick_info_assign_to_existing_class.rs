use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_assign_to_existing_class() {
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
    let mut s = Session::new_for_test("quickInfoAssignToExistingClass", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyQuickInfoExists(t)
}
