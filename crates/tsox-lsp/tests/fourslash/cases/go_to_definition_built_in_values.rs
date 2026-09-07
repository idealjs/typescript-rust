use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_built_in_values() {
    let content = r#"var u = /*undefined*/undefined;
var n = /*null*/null;
var a = function() { return /*arguments*/arguments; };
var t = /*true*/true;
var f = /*false*/false;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, f.MarkerNames()...)
}
