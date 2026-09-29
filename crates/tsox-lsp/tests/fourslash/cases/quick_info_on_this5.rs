use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_on_this5() {
    let content = r#"// @noImplicitThis: true
const foo = {
    num: 0,
    f() {
        type Y = typeof th/*1*/is;
        type Z = typeof th/*2*/is.num;
    },
    g(this: number) {
        type X = typeof th/*3*/is;
    }
}
class Foo {
    num = 0;
    f() {
        type Y = typeof th/*4*/is;
        type Z = typeof th/*5*/is.num;
    }
    g(this: number) {
        type X = typeof th/*6*/is;
    }
}"#;
    let _s = Session::new_for_test("quickInfoOnThis5", content);
    // TODO: f.VerifyBaselineHover(t)
}
