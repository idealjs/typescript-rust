use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn return_recursive_type() {
    let content = r#"interface MyInt {
    (): void;
}
function MyFn() { return <MyInt>MyFn; }
var My/**/Var = MyFn();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "var MyVar: MyInt", "")
}
