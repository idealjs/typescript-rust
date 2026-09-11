use tsox_lsp::fourslash::{self, Session};


#[test]
fn generic_interface_with_inheritance_edit1() {
    let content = r#"interface ChainedObject<T> {
    values(): ChainedArray<any>;
    pairs(): ChainedArray<any[]>;
    extend(...sources: any[]): ChainedObject<T>;

    value(): T;
}
interface ChainedArray<T> extends ChainedObject<Array<T>> {

    extend(...sources: any[]): ChainedArray<T>;
}
 /*1*/"#;
    let mut s = Session::new_for_test("genericInterfaceWithInheritanceEdit1", content);
    fourslash::verify_no_errors(&mut s, );
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, " ");
    fourslash::verify_no_errors(&mut s, );
}
