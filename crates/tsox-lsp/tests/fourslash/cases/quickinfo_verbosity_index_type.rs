use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_index_type() {
    let content = r#"interface T1 {
	banana: string;
	grape: number;
	apple: boolean;
}
const x1/*x1*/: keyof T1 = 'banana';
const x2/*x2*/: keyof T1 & ("grape" | "apple") = 'grape';
function fn1<T extends T1>(obj: T, key: keyof T, k2: keyof T1) {
	if (key === k2/*k2*/) {
		return obj[key/*key*/];
	}
	return key;
}"#;
    let mut s = Session::new_for_test("quickinfoVerbosityIndexType", content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"x1": {0, 1}, "x2": {0}, "k2": {0, 1}, "key":
}
