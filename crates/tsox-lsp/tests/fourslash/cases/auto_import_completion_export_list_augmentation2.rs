use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
#[test]
fn auto_import_completion_export_list_augmentation2() {
    let content = r#"// @module: node18
// @Filename: /node_modules/@sapphire/pieces/index.d.ts
interface Container {
  stores: unknown;
}

declare class Piece {
  get container(): Container;
}

declare class AliasPiece extends Piece {}

export { AliasPiece, type Container };
// @Filename: /node_modules/@sapphire/framework/index.d.ts
import { AliasPiece } from "@sapphire/pieces";

declare class Command extends AliasPiece {}

declare module "@sapphire/pieces" {
  interface Container {
    client: unknown;
  }
}

export { Command };
// @Filename: /index.ts
import "@sapphire/pieces";
import { Command } from "@sapphire/framework";
class PingCommand extends Command {
  /*1*/
}"#;
    let mut s = Session::new_for_test("autoImportCompletionExportListAugmentation2", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new("1"), &fourslash.ApplyCodeActionFromCompletionOptions{
}
