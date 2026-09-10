use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_overriding_method4() {
    let content = r#"// @newline: LF
// @Filename: secret.ts
class Secret {
    #secret(): string {
        return "secret";
    }

    private tell(): string {
        return this.#secret();
    }

    protected hint(): string {
        return "hint";
    }

    public refuse(): string {
        return "no comments";
    }
}

class Gossip extends Secret {
    /* no telling secrets */
    /*a*/
}"#;
    let mut s = Session::new_for_test("completionsOverridingMethod4", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}
