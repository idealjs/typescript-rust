use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
#[test]
fn auto_import_completion_export_list_augmentation1() {
    let content = r#"// @module: node18
// @Filename: /node_modules/@sapphire/pieces/index.d.ts
interface Container {
  stores: unknown;
}

declare class Piece {
  container: Container;
}

export { Piece, type Container };
// @FileName: /augmentation.ts
declare module "@sapphire/pieces" {
  interface Container {
    client: unknown;
  }
  export { Container };
}
// @Filename: /index.ts
import { Piece } from "@sapphire/pieces";
class FullPiece extends Piece {
  /*1*/
}"#;
    let mut s = Session::new_for_test("autoImportCompletionExportListAugmentation1", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new("1"), &fourslash.ApplyCodeActionFromCompletionOptions{
}
