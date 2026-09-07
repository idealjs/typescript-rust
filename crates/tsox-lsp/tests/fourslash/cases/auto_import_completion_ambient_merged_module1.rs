use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
#[test]
fn auto_import_completion_ambient_merged_module1() {
    let content = r#"// @strict: true
// @module: commonjs
// @filename: /node_modules/@types/vscode/index.d.ts
declare module "vscode" {
  export class Position {
    readonly line: number;
    readonly character: number;
  }
}
// @filename: src/motion.ts
import { Position } from "vscode";

export abstract class MoveQuoteMatch {
  public override async execActionWithCount(
    position: Position,
  ): Promise<void> {}
}

declare module "vscode" {
  interface Position {
    toString(): string;
  }
}
// @filename: src/smartQuotes.ts
import { MoveQuoteMatch } from "./motion";

export class MoveInsideNextQuote extends MoveQuoteMatch {/*1*/
  keys = ["i", "n", "q"];
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new("1"), &fourslash.ApplyCodeActionFromCompletionOptions{
}
