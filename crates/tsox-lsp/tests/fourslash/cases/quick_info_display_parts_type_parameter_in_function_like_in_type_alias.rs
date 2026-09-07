use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_display_parts_type_parameter_in_function_like_in_type_alias() {
    let content = r#"type MixinCtor<A> = new () => /*0*/A & { constructor: MixinCtor</*1*/A> };
type MixinCtor<A> = new () => A & { constructor: { constructor: MixinCtor</*2*/A> } };"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
