use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_function_parameter1() {
    let content = r#"function Foo() {
    /**
     * @param {number} p
     */
    this.foo = function foo(p/**/) {
        return p;
    }
}"#;
    let mut s = Session::new_for_test("renameFunctionParameter1", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
