use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_overridden_member17() {
    let content = r#"// @strict: true
// @target: esnext
// @lib: esnext
const entityKind = Symbol.for("drizzle:entityKind");

abstract class MySqlColumn {
  static readonly /*2*/[entityKind]: string = "MySqlColumn";
}

export class MySqlVarBinary extends MySqlColumn {
  static [|/*1*/override|] readonly [entityKind]: string = "MySqlVarBinary";
}"#;
    let mut s = Session::new_for_test("goToDefinitionOverriddenMember17", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
