use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_on_decorators() {
    let content = r#"// @Filename: a.ts
/*1*/function /*2*/decorator(target) {
    return target;
}
/*3*/decorator();
// @Filename: b.ts
@/*4*/decorator @/*5*/decorator("again")
class C {
    @/*6*/decorator
    method() {}
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6")
}
