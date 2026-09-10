use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToTypeDefinition"]
#[test]
fn go_to_type_definition_array_type() {
    let content = r#"// @lib: es5
type User = { name: string };
declare const users: User[]
/*reference*/users

type UsersArr = Array<User>
declare const users2: UsersArr
/*reference2*/users2

class CustomArray<T> extends Array<T> { immutableReverse() { return [...this].reverse() } }
declare const users3: CustomArray<User>
/*reference3*/users3"#;
    let mut s = Session::new_for_test("goToTypeDefinition_arrayType", content);
    fourslash::unsupported("VerifyBaselineGoToTypeDefinition"); // f.VerifyBaselineGoToTypeDefinition(t, "reference", "reference2", "reference3")
}
