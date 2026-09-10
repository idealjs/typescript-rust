use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("completionListAtNodeBoundary", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["charAt"], &[]);
}
