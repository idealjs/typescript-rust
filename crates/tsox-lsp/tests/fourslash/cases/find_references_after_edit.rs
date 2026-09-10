use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_references_after_edit() {
    let content = r#"// @Filename: a.ts
interface A {
    /*1*/foo: string;
}
// @Filename: b.ts
///<reference path='a.ts'/>
/**/
function foo(x: A) {
    x./*2*/foo
}"#;
    let mut s = Session::new_for_test("findReferencesAfterEdit", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2")
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "\n");
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2")
}
