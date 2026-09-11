use tsox_lsp::fourslash::{self, Session};


#[test]
fn quickinfo_verbosity_class_with_mixin_base() {
    let content = r#"
class Base {}

declare const Mixin: new () => Base & { mixed: string };

class Derived/*1*/ extends Mixin {}
"#;
    let mut s = Session::new_for_test("quickinfoVerbosityClassWithMixinBase", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
