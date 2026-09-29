use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_deprecated_suggestion8() {
    let content = r#"// @Filename: first.ts
/** @deprecated */
export declare function tap<T>(next: null): void;
export declare function tap<T>(next: T): T;
// @Filename: second.ts
import { tap } from './first';
tap"#;
    let mut s = Session::new_for_test("jsdocDeprecated_suggestion8", content);
    fourslash::go_to_file(&mut s, "second.ts");
    fourslash::verify_no_errors(&mut s, );
    // TODO: f.VerifySuggestionDiagnostics(t, nil)
}
