use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineCodeLens"]
#[test]
fn code_lens_overloads01() {
    let content = r#"
export function foo(x: number): number;
export function foo(x: string): string;
export function foo(x: string | number): string | number {
	return x;
}

foo(1);

foo("hello");

// This one isn't expected to match any overload,
// but is really just here to test how it affects how code lens.
foo(Math.random() ? 1 : "hello");
"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineCodeLens"); // f.VerifyBaselineCodeLens(t, &lsutil.UserPreferences{
}
