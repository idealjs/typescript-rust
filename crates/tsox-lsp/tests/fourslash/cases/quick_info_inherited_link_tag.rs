use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_inherited_link_tag() {
    let content = r#"export class C {
     /**
      * @deprecated Use {@link PerspectiveCamera#setFocalLength .setFocalLength()} and {@link PerspectiveCamera#filmGauge .filmGauge} instead.
      */
    m() { }
}
export class D extends C {
    m() { } // crashes here
}
new C().m/**/ // and here (with a different thing trying to access undefined)"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
