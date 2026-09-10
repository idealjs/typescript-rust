use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_lib_type() {
    let content = r#"// @lib: es5
interface Apple {
    color: string;
    size: number;
}
function f(): Promise<Apple> {
    return Promise.resolve({ color: "red", size: 5 });
}
const g/*g*/ = f;
const u/*u*/: Map<string, Apple> = new Map;
type Foo<T> = Promise/*p*/<T>;"#;
    let mut s = Session::new_for_test("quickinfoVerbosityLibType", content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"g": {0, 1}, "u": {0, 1}, "p": {0}})
}
