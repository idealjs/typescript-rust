use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNoErrors"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, " ");
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
