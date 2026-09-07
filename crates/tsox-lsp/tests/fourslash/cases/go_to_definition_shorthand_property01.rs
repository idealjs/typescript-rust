use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_shorthand_property01() {
    let content = r#"// @lib: es5
var /*valueDeclaration1*/name = "hello";
var /*valueDeclaration2*/id = 100000;
declare var /*valueDeclaration3*/id;
var obj = {[|/*valueDefinition1*/name|], [|/*valueDefinition2*/id|]};
obj.[|/*valueReference1*/name|];
obj.[|/*valueReference2*/id|];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "valueDefinition1", "valueDefinition2", "valueReference1", "
}
