use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
#[test]
fn auto_import_completion_export_list_augmentation3() {
    let content = r#"// @module: node18
// @Filename: /node_modules/@sapphire/pieces/index.d.ts
export interface Container {
  stores: unknown;
}

declare class Piece {
  container: Container;
}

export { Piece };
// @FileName: /augmentation.ts
declare module "@sapphire/pieces" {
  interface Container {
    client: unknown;
  }
}
// @Filename: /index.ts
import { Piece } from "@sapphire/pieces";
class FullPiece extends Piece {
  /*1*/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new("1"), &fourslash.ApplyCodeActionFromCompletionOptions{
}
