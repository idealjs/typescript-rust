use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_with_shorthand_property_assignment() {
    let content = r#"// @lib: es5
var /*0*/name = "Foo";

var obj = { /*1*/name };
var obj1 = { /*2*/name: /*3*/name };
obj./*4*/name;"#;
    let mut s = Session::new_for_test("findAllRefsWithShorthandPropertyAssignment", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0", "3", "1", "2", "4")
}
