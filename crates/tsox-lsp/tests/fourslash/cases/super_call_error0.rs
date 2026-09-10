use tsox_lsp::fourslash::{self, Session};


#[test]
fn super_call_error0() {
    let content = r#"class T5<T>{
    constructor(public bar: T) { }
}
class T6 extends T5<number>{
    constructor() {
        super();
    }
}/*1*/"#;
    let mut s = Session::new_for_test("superCallError0", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "/n");
}
