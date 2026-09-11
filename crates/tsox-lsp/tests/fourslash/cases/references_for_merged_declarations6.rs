use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_for_merged_declarations6() {
    let content = r#"interface Foo { }
/*1*/module /*2*/Foo {
    export interface Bar { }
    export namespace Bar { export interface Baz { } }
    export function Bar() { }
}

// module
import a1 = /*3*/Foo;"#;
    let mut s = Session::new_for_test("referencesForMergedDeclarations6", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
