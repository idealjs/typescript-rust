use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_write_access() {
    let content = r#"interface Obj {
    [`/*1*/num`]: number;
}

let o: Obj = {
    [`num`]: 0
};

o = {
    ['num']: 1
};

o['num'] = 2;
o[`num`] = 3;

o['num'];
o[`num`];"#;
    let mut s = Session::new_for_test("findAllRefsWriteAccess", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
