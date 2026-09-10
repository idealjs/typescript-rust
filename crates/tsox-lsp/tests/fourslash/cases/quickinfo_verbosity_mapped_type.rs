use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_mapped_type() {
    let content = r#"type Apple = boolean | number;
type Orange = string | boolean;
type F<T> = {
	[K in keyof T as T[K] extends Apple ? never : K]: T[K];
}
type Bar = {
	banana: string;
	apple: boolean;
}
const x/*x*/: F/*F*/<Bar> = { banana: 'hello' };
const y/*y*/: { [K in keyof Bar]?: Bar[K] } = { banana: 'hello' };
type G<T> = {
	[K in keyof T]: T[K] & Apple
};
const z: G/*G*/<Bar> = { banana: 'hello', apple: true };"#;
    let mut s = Session::new_for_test("quickinfoVerbosityMappedType", content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"x": {0, 1}, "y": {0}, "F": {0, 1}, "G": {0, 
}
