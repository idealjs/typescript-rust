use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_references_link_tag2() {
    let content = r#"namespace NPR/*5*/ {
    export class Consider/*4*/ {
        This/*3*/ = class {
            show/*2*/() { }
        }
        m/*1*/() { }
    }
    /**
     * @see {Consider.prototype.m}
     * {@link Consider#m}
     * @see {Consider#This#show}
     * {@link Consider.This.show}
     * @see {NPR.Consider#This#show}
     * {@link NPR.Consider.This#show}
     * @see {NPR.Consider#This.show} # doesn't parse trailing .
     * @see {NPR.Consider.This.show}
     */
    export function ref() { }
}
/**
 * {@link NPR.Consider#This#show hello hello}
 * {@link NPR.Consider.This#show}
 * @see {NPR.Consider#This.show} # doesn't parse trailing .
 * @see {NPR.Consider.This.show}
 */
export function outerref() { }"#;
    let mut s = Session::new_for_test("findAllReferencesLinkTag2", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5")
}
