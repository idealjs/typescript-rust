use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_built_in_types() {
    let content = r#"var n: /*number*/number;
var s: /*string*/string;
var b: /*boolean*/boolean;
var v: /*void*/void;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, f.MarkerNames()...)
}
