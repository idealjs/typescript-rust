use tsox_lsp::fourslash::{self, Session};


#[test]
fn inlay_hints_element_access() {
    let content = r#"interface MySymbol {
	readonly "my dispose": unique symbol
}

declare var mySymbol: MySymbol;

let foo = {
	[mySymbol["my dispose"]]: () => {}
}
"#;
    let mut s = Session::new_for_test("inlayHintsElementAccess", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
