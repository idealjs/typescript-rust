use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_references_jsx_tag_name2() {
    let content = r#"// @Filename: index.tsx
/*1*/const /*2*/obj = {Component: () => <div/>};
const element = </*3*/obj.Component/>;"#;
    let mut s = Session::new_for_test("findReferencesJSXTagName2", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
