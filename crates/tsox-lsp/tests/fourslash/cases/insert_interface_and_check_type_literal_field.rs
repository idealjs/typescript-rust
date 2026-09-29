use tsox_lsp::fourslash::{self, Session};


#[test]
fn insert_interface_and_check_type_literal_field() {
    let content = r#"/*addC*/
interface G<T, U> { }
var v2: G<{ a: /*checkParam*/C }, C>;"#;
    let mut s = Session::new_for_test("insertInterfaceAndCheckTypeLiteralField", content);
    fourslash::go_to_marker(&mut s, "addC");
    fourslash::insert(&mut s, "interface C { }");
    fourslash::go_to_marker(&mut s, "checkParam");
    // TODO: f.VerifyQuickInfoExists(t)
}
