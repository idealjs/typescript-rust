use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: opts786 := f.GetOptions()"]
#[test]
fn completion_class_member_snippet_cross_file_node_reuse1() {
    let content = r#"// @strict: true
// @filename: KlassConstructor.ts
type GenericConstructor<T> = new (...args: any[]) => T;
export type KlassConstructor<Cls extends GenericConstructor<any>> =
  GenericConstructor<InstanceType<Cls>> & { [k in keyof Cls]: Cls[k] };
// @filename: ElementNode.ts
import { KlassConstructor } from "./KlassConstructor";

export type NodeKey = string;

export class ElementNode {
  ["constructor"]!: KlassConstructor<typeof ElementNode>;
}
// @filename: CollapsibleContainerNode.ts
import { ElementNode, NodeKey } from "./ElementNode";

export class CollapsibleContainerNode extends ElementNode {
  __open: boolean;

  /*1*/
}"#;
    let mut s = Session::new(content);
    // TODO: opts786 := f.GetOptions()
    // TODO: opts786.FormatCodeSettings.InsertSpaceAfterConstructor = core.TSFalse
    fourslash::unsupported("Configure"); // f.Configure(t, opts786)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
