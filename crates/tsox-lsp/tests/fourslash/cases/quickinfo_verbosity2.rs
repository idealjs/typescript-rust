use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity2() {
    let content = r#"type Str = string | {};
type FooType = Str | number;
type Sym = symbol | (() => void);
type BarType = Sym | boolean;
type BothType = FooType | BarType;
const both/*b*/: BothType = 1;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"b": {0, 1, 2, 3}})
}
