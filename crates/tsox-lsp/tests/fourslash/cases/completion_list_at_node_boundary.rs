use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_at_node_boundary() {
    let content = r#"interface Iterator<T, U> {
    (value: T, index: any, list: any): U;
}

interface WrappedArray<T> {
    map<U>(iterator: Iterator<T, U>, context?: any): U[];
}

interface Underscore {
    <T>(list: T[]): WrappedArray<T>;
    map<T, U>(list: T[], iterator: Iterator<T, U>, context?: any): U[];
}

declare var _: Underscore;
var a: string[];
var e = a.map(x => x./**/);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
