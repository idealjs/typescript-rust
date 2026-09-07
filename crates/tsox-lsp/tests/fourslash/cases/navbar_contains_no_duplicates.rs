use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
