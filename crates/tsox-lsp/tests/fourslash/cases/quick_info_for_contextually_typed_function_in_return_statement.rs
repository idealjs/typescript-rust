use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_for_contextually_typed_function_in_return_statement() {
    let content = r#"interface Accumulator {
    clear(): void;
    add(x: number): void;
    result(): number;
}

function makeAccumulator(): Accumulator {
    var sum = 0;
    return {
        clear: function () { sum = 0; },
        add: function (val/**/ue) { sum += value; },
        result: function () { return sum; }
    };
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "(parameter) value: number", "")
}
