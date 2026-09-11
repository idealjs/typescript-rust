use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_getter_returning_callable_interface() {
    let content = r#"// @Filename: /home/src/workspaces/project/type.d.ts
export interface DidChangeContentEvent {
    (): void;
}

export declare class TextDocuments {
    get onDidChangeContent(): DidChangeContentEvent;
}

// @Filename: /home/src/workspaces/project/index.ts
import { TextDocuments } from "./type";

declare const documents: TextDocuments | undefined;

documents!./*usage*/onDidChangeContent()"#;
    let mut s = Session::new_for_test("goToDefinitionGetterReturningCallableInterface", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, false /*includeOriginalSelectionRange*/, "usage")
}
