use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineInlayHints"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineInlayHints"); // f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
