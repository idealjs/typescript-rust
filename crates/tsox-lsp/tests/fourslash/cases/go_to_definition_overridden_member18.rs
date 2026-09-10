use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_overridden_member18() {
    let content = r#"// @strict: true
// @target: esnext
// @lib: esnext
const entityKind = Symbol.for("drizzle:entityKind");

abstract class MySqlColumn {
  readonly /*2*/[entityKind]: string = "MySqlColumn";
}

export class MySqlVarBinary extends MySqlColumn {
  [|/*1*/override|] readonly [entityKind]: string = "MySqlVarBinary";
}"#;
    let mut s = Session::new_for_test("goToDefinitionOverriddenMember18", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
