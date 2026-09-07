use tsox_lsp::fourslash::{self, Session};

#[test]
fn return_recursive_type() {
    let content = r#"interface MyInt {
    (): void;
}
function MyFn() { return <MyInt>MyFn; }
var My/**/Var = MyFn();"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "", "var MyVar: MyInt", "");
}
