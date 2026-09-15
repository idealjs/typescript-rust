use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("navigationBarItemsModules1", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
