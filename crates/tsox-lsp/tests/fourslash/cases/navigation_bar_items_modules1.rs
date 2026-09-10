use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_items_modules1() {
    let content = r#"declare module "X.Y.Z" {}

declare module 'X2.Y2.Z2' {}

declare module "foo";

namespace A.B.C {
    export var x;
}

namespace A.B {
    export var y;
}

namespace A {
    export var z;
}

namespace A {
    namespace B {
        namespace C {
            declare var x;
        }
    }
}"#;
    let mut s = Session::new_for_test("navigationBarItemsModules1", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
