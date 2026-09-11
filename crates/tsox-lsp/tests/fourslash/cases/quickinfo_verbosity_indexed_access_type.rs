use tsox_lsp::fourslash::{self, Session};


#[test]
fn quickinfo_verbosity_indexed_access_type() {
    let content = r#"interface T2 {
	"string key": string;
	"number key": number;
	"any key": string | number | symbol;
}
type K2 = "string key" | "any key";
function fn2<T extends T2>(obj: T, key: keyof T) {
	const value/*v1*/: T[K2] = undefined as any;
}
function fn3<K extends keyof T2>(obj: T2, key: K) {
    const value/*v2*/: T2[K] = undefined as any;;
}"#;
    let mut s = Session::new_for_test("quickinfoVerbosityIndexedAccessType", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"v1": {0, 1}, "v2": {0, 1}})
}
