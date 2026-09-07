use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.GoToPosition"]
#[test]
fn underscore_typings02() {
    let content = r#"// @strict: false
// @module: CommonJS
interface Dictionary<T> {
    [x: string]: T;
}
export interface ChainedObject<T> {
    functions: ChainedArray<string>;
    omit(): ChainedObject<T>;
    clone(): ChainedObject<T>;
}
interface ChainedDictionary<T> extends ChainedObject<Dictionary<>> {
    foldl(): ChainedObject<T>;
    clone(): ChainedDictionary<T>;
}
export interface ChainedArray<T> extends ChainedObject<Array<T>> {
    groupBy(): ChainedDictionary<any[]>;
    groupBy(propertyName): ChainedDictionary<any[]>;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("GoToPosition"); // f.GoToPosition(t, 0)
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 2)
}
