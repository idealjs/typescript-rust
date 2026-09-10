use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_for_signature_with_unreachable_type() {
    let content = r#"// @Filename: /node_modules/foo/node_modules/bar/index.d.ts
export interface SomeType {
    x?: number;
}
// @Filename: /node_modules/foo/index.d.ts
import { SomeType } from "bar";
export function func<T extends SomeType>(param: T): void;
export function func<T extends SomeType>(param: T, other: T): void;
// @Filename: /usage.ts
import { func } from "foo";
func({/*1*/});"#;
    let mut s = Session::new_for_test("signatureHelpForSignatureWithUnreachableType", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "func(param: {}): void", Overloa
}
