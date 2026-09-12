use tsox_lsp::fourslash::{self, Session};


#[test]
fn inlay_hints_interactive_template_literal_types() {
    let content = r#"declare function getTemplateLiteral1(): `${string},${string}`;
const lit1 = getTemplateLiteral1();
declare function getTemplateLiteral2(): `\${${string},${string}`;
const lit2 = getTemplateLiteral2();
declare function getTemplateLiteral3(): `start${string}\${,$${string}end`;
const lit3 = getTemplateLiteral3();
declare function getTemplateLiteral4(): `${string}\`,${string}`;
const lit4 = getTemplateLiteral4();"#;
    let mut s = Session::new_for_test("inlayHintsInteractiveTemplateLiteralTypes", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
    // TODO: }
}
