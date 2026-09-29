use tsox_lsp::fourslash::Session;


#[test]
fn signature_help_tagged_templates_negatives1() {
    let content = r#"function f(templateStrings, x, y, z) { return 10; }
function g(templateStrings, x, y, z) { return ""; }

/*1*/f/*2*/ /*3*/` qwerty ${ 123 } asdf ${   41234   }  zxcvb ${ g `    ` }     `/*4*/"#;
    let _s = Session::new_for_test("signatureHelpTaggedTemplatesNegatives1", content);
    // TODO: f.VerifyNoSignatureHelpForMarkers(t, f.MarkerNames()...)
}
