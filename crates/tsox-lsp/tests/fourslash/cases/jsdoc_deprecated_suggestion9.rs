use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_deprecated_suggestion9() {
    let content = r#"// @Filename: first.ts
export class logger { }
// @Filename: second.ts
import { logger } from './first';
new logger()"#;
    let mut s = Session::new_for_test("jsdocDeprecated_suggestion9", content);
    fourslash::go_to_file(&mut s, "second.ts");
    fourslash::verify_no_errors(&mut s, );
    // TODO: f.VerifySuggestionDiagnostics(t, nil)
}
