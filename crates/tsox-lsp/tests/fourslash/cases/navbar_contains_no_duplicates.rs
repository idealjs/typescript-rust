use tsox_lsp::fourslash::Session;


#[test]
fn navbar_contains_no_duplicates() {
    let content = r#"declare namespace Windows {
    export namespace Foundation {
        export var A;
        export class Test {
            public wow();
        }
    }
}

declare namespace Windows {
    export namespace Foundation {
        export var B;
        export namespace Test {
            export function Boom(): number;
        }
    }
}

class ABC {
    public foo() {
        return 3;
    }
}

namespace ABC {
    export var x = 3;
}"#;
    let _s = Session::new_for_test("navbar_contains_no_duplicates", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
