use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNoSignatureHelpForMarkers"]
#[test]
fn signature_help_tagged_templates_negatives1() {
    let content = r#"function f(templateStrings, x, y, z) { return 10; }
function g(templateStrings, x, y, z) { return ""; }

/*1*/f/*2*/ /*3*/` + "`" + ` qwerty ${ 123 } asdf ${   41234   }  zxcvb ${ g ` + "`" + `    ` + "`" + ` }     ` + "`" + `/*4*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoSignatureHelpForMarkers"); // f.VerifyNoSignatureHelpForMarkers(t, f.MarkerNames()...)
}
