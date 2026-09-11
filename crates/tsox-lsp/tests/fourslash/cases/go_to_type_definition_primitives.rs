use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_type_definition_primitives() {
    let content = r#"// @Filename: module1.ts
var w: {a: number};
var x = "string";
var y: number | string;
var z; // any
// @Filename: module2.ts
w./*reference1*/a;
/*reference2*/x;
/*reference3*/y;
/*reference4*/y;"#;
    let mut s = Session::new_for_test("goToTypeDefinitionPrimitives", content);
    // TODO: f.VerifyBaselineGoToTypeDefinition(t, "reference1", "reference2", "reference3", "reference4")
}
