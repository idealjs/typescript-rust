use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_type_parameter() {
    let content = r#"type Str = string | {};
type FooType = Str | number;
function fn<T extends FooType>(x: T) {
    x/*x*/;
}
const y/*y*/: <T extends FooType>(x: T) => void = fn;
type MixinCtor<A> = new () => A/*a*/ & { constructor: MixinCtor<A> };"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"x": {0, 1, 2}, "y": {0, 1, 2}, "a": {0}})
}
