use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_recursive_type() {
    let content = r#"// @lib: es5
type Node/*N*/<T> = {
    value: T;
    left: Node<T> | undefined;
    right: Node<T> | undefined;
}
const n/*n*/: Node<number> = {
    value: 1,
    left: undefined,
    right: undefined,
}
interface Orange {
    name: string;
}
type TreeNode/*t*/<T> = {
    value: T;
    left: TreeNode<T> | undefined;
    right: TreeNode<T> | undefined;
    orange?: Orange;
}
const m/*m*/: TreeNode<number> = {
    value: 1,
    left: undefined,
    right: undefined,
    orange: { name: "orange" },
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"N": {0}, "n": {0, 1}, "t": {0, 1}, "m": {0, 
}
