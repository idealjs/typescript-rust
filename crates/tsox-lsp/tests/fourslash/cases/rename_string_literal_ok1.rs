use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn rename_string_literal_ok1() {
    let content = r#"declare function f(): '[|foo|]' | 'bar'
class Foo {
    f = f()
}
const d: 'foo' = 'foo'
declare const ff: Foo
ff.f = '[|foo|]'"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "foo")
}
