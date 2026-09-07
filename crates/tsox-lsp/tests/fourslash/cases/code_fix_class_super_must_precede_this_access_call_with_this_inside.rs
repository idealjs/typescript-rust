use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_class_super_must_precede_this_access_call_with_this_inside() {
    let content = r#"class Base{
    constructor(id: number) { id; }
}
class C extends Base{
    constructor(private a:number) {
        super(this.a);
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}
