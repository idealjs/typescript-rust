use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_link_code_plain() {
    let content = r#"export class C {
     /**
      * @deprecated Use {@linkplain PerspectiveCamera#setFocalLength .setFocalLength()} and {@linkcode PerspectiveCamera#filmGauge .filmGauge} instead.
      */
    m() { }
}
new C().m/**/"#;
    let mut s = Session::new_for_test("quickInfoLinkCodePlain", content);
    fourslash::verify_no_errors(&mut s, );
    // TODO: f.VerifyBaselineHover(t)
}
