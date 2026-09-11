use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_string_literal_ok1() {
    let content = r#"declare function f(): '[|foo|]' | 'bar'
class Foo {
    f = f()
}
const d: 'foo' = 'foo'
declare const ff: Foo
ff.f = '[|foo|]'"#;
    let mut s = Session::new_for_test("renameStringLiteralOk1", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "foo")
}
