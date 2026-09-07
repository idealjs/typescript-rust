use tsox_lsp::fourslash::{self, Session};

#[test]
fn update_to_class_statics() {
    let content = r#"namespace TypeScript {
    export class PullSymbol {}
    export class Diagnostic {}
    export class SymbolAndDiagnostics<TSymbol extends PullSymbol> {
        constructor(public symbol: TSymbol,
            public diagnostics: Diagnostic) {
        }
        /**/
        public static create<TSymbol extends PullSymbol>(symbol: TSymbol, diagnostics: Diagnostic): SymbolAndDiagnostics<TSymbol> {
            return new SymbolAndDiagnostics<TSymbol>(symbol, diagnostics);
        }
    }
}
namespace TypeScript {
    var x : TypeScript.SymbolAndDiagnostics;
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "someNewProperty = 0;");
}
