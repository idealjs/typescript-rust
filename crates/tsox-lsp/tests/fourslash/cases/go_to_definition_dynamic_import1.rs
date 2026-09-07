use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_dynamic_import1() {
    let content = r#"// @Filename: foo.ts
/*Destination*/export function foo() { return "foo"; }
import([|"./f/*1*/oo"|])
var x = import([|"./fo/*2*/o"|])"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1", "2")
}
