use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_overriding_method_crash2() {
    let content = r#"// @Filename: /home/src/workspaces/project/tsconfig.json
{
  "compilerOptions": {
    "module": "nodenext",
    "lib": ["es5"]
  }
}
// @Filename: /home/src/workspaces/project/utils.ts
export class Element {
    // ...
}

export abstract class Component {
    abstract render(): Element;
}
// @Filename: /home/src/workspaces/project/classes.ts
import { Component } from "./utils.js";

export class MyComponent extends Component {
    [|render/**/|]
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::go_to_file(&mut s, "/home/src/workspaces/project/utils.ts");
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.Backspace(t, 1)
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
