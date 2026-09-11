use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_completion_export_equals_with_default1() {
    let content = r#"// @strict: true
// @module: commonjs
// @esModuleInterop: false
// @allowSyntheticDefaultImports: false
// @filename: node.ts
import Container from "./container.js";
import Document from "./document.js";

declare namespace Node {
  class Node extends Node_ {}

  export { Node as default };
}

declare abstract class Node_ {
  parent: Container | Document | undefined;
}

declare class Node extends Node_ {}

export = Node;
// @filename: document.ts
import Container from "./container.js";

declare namespace Document {
  export { Document_ as default };
}

declare class Document_ extends Container {}

declare class Document extends Document_ {}

export = Document;
// @filename: container.ts
import Node from "./node.js";

declare namespace Container {
  export { Container_ as default };
}

declare abstract class Container_ extends Node {
  p/*1*/
}

declare class Container extends Container_ {}

export = Container;"#;
    let mut s = Session::new_for_test("autoImportCompletionExportEqualsWithDefault1", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new("1"), &fourslash.ApplyCodeActionFromCompletionOptions{
}
