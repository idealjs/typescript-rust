use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_completion_export_list_augmentation4() {
    let content = r#"// @module: node18
// @Filename: /node_modules/@sapphire/pieces/index.d.ts
interface Container {
  stores: unknown;
}

declare class Piece {
  get container(): Container;
}

export { Piece as Alias, type Container };
// @Filename: /node_modules/@sapphire/framework/index.d.ts
import { Alias } from "@sapphire/pieces";

declare class Command extends Alias {}

declare module "@sapphire/pieces" {
  interface Container {
    client: unknown;
  }
}

export { Command as CommandAlias };
// @Filename: /index.ts
import "@sapphire/pieces";
import { CommandAlias } from "@sapphire/framework";
class PingCommand extends CommandAlias {
  /*1*/
}"#;
    let mut s = Session::new_for_test("autoImportCompletionExportListAugmentation4", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new("1"), &fourslash.ApplyCodeActionFromCompletionOptions{
}
