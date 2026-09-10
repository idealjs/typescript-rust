use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_this_keyword_multiple_files() {
    let content = r#"// @Filename: file1.ts
/*1*/this; /*2*/this;
// @Filename: file2.ts
/*3*/this;
/*4*/this;
// @Filename: file3.ts
 ((x = /*5*/this, y) => /*6*/this)(/*7*/this, /*8*/this);
 // different 'this'
 function f(this) { return this; }"#;
    let mut s = Session::new_for_test("findAllRefsThisKeywordMultipleFiles", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6", "7", "8")
}
