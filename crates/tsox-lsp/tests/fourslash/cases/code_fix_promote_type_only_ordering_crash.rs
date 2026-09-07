use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn code_fix_promote_type_only_ordering_crash() {
    let content = r#"// @module: node18
// @verbatimModuleSyntax: true
// @Filename: /bar.ts
export interface AAA {}
export class BBB {}
// @Filename: /foo.ts
import type {
    AAA,
    BBB,
} from "./bar";

let x: AAA = new BBB()"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/foo.ts");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
