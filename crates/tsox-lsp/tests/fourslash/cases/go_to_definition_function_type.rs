use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_function_type() {
    let content = r#"const /*constDefinition*/c: () => void;
/*constReference*/c();
function test(/*cbDefinition*/cb: () => void) {
    /*cbReference*/cb();
}
class C {
    /*propDefinition*/prop: () => void;
    m() {
        this./*propReference*/prop();
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, false, "constReference", "cbReference", "propReference")
}
