use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn generic_type_alias_intersection_completions() {
    let content = r#"type MixinCtor<A, B> = new () => A & B & { constructor: MixinCtor<A, B> };
function merge<A, B>(a: { prototype: A }, b: { prototype: B }): MixinCtor<A, B> {
  let merged = function() { }
  Object.assign(merged.prototype, a.prototype, b.prototype);
  return <MixinCtor<A, B>><any>merged;
}

class TreeNode {
  value: any;
}

abstract class LeftSideNode extends TreeNode {
  abstract right(): TreeNode;
  left(): TreeNode {
    return null;
  }
}

abstract class RightSideNode extends TreeNode {
  abstract left(): TreeNode;
  right(): TreeNode {
    return null;
  };
}

var obj = new (merge(LeftSideNode, RightSideNode))();
obj./**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
