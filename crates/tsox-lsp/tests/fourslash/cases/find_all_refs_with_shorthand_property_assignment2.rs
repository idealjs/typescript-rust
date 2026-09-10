use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_with_shorthand_property_assignment2() {
    let content = r#"var /*0*/dx = "Foo";

namespace M { export var /*1*/dx; }
namespace M {
   var z = 100;
   export var y = { /*2*/dx, z };
}
M.y./*3*/dx;"#;
    let mut s = Session::new_for_test("findAllRefsWithShorthandPropertyAssignment2", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0", "1", "2", "3")
}
