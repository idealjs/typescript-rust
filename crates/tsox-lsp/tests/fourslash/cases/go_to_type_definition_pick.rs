use tsox_lsp::fourslash::Session;


#[test]
fn go_to_type_definition_pick() {
    let content = r#"// @lib: es5
type User = { id: number; name: string; };
declare const user: Pick<User, "name">
/*reference*/user

type PickedUser = Pick<User, "name">
declare const user2: PickedUser
/*reference2*/user2"#;
    let _s = Session::new_for_test("goToTypeDefinition_Pick", content);
    // TODO: f.VerifyBaselineGoToTypeDefinition(t, "reference", "reference2")
}
