use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNoErrors"]
#[test]
fn jsdoc_deprecated_suggestion9() {
    let content = r#"// @Filename: first.ts
export class logger { }
// @Filename: second.ts
import { logger } from './first';
new logger()"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "second.ts");
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, nil)
}
