use tsox_lsp::fourslash::{self, Session};


#[test]
fn generic_interfaces_with_constraints1() {
    let content = r#"interface A { a: string; }
interface B extends A { b: string; }
interface C extends B { c: string; }
interface G<T, U extends B> {
    x: T;
    y: U;
}
var v/*1*/1: G<A, C>;               // Ok
var v/*2*/2: G<{ a: string }, C>;   // Ok, equivalent to G<A, C>
var v/*3*/3: G<G<A, B>, C>;         // Ok"#;
    let mut s = Session::new_for_test("genericInterfacesWithConstraints1", content);
    fourslash::verify_quick_info_at(&mut s, "1", "var v1: G<A, C>", "");
    fourslash::verify_quick_info_at(&mut s, "2", "var v2: G<{\n    a: string;\n}, C>", "");
    fourslash::verify_quick_info_at(&mut s, "3", "var v3: G<G<A, B>, C>", "");
}
