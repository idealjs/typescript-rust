use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToTypeDefinition"]
#[test]
fn go_to_type_definition_pick() {
    let content = r#"// @lib: es5
type User = { id: number; name: string; };
declare const user: Pick<User, "name">
/*reference*/user

type PickedUser = Pick<User, "name">
declare const user2: PickedUser
/*reference2*/user2"#;
    let mut s = Session::new_for_test("goToTypeDefinition_Pick", content);
    fourslash::unsupported("VerifyBaselineGoToTypeDefinition"); // f.VerifyBaselineGoToTypeDefinition(t, "reference", "reference2")
}
