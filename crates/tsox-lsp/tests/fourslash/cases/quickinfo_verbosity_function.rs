use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_function() {
    let content = r#"interface Apple {
    color: string;
    size: number;
}
interface Orchard {
    takeOneApple(a: Apple): void;
    getApple(): Apple;
    getApple(size: number): Apple[];
}
const o/*o*/: Orchard = {} as any;
declare function isApple/*f*/(x: unknown): x is Apple;
type SomeType = {
    prop1: string;
}
function someFun(a: SomeType): SomeType {
    return a;
}
someFun/*s*/.what = 'what';"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"o": {0, 1, 2}, "f": {0, 1}, "s": {0, 1}})
}
