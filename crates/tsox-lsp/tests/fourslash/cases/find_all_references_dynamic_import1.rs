use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_references_dynamic_import1() {
    let content = r#"// @lib: es5
// @Filename: foo.ts
export function foo() { return "foo"; }
/*1*/import("/*2*/./foo")
/*3*/var x = import("/*4*/./foo")"#;
    let mut s = Session::new_for_test("findAllReferencesDynamicImport1", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
