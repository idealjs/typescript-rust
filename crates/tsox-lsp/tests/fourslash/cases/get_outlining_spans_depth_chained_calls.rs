use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOutliningSpans"]
#[test]
fn get_outlining_spans_depth_chained_calls() {
    let content = r#"declare var router: any;
router
    .get[|("/", async(ctx) =>[|{
        ctx.body = "base";
    }|])|]
    .post[|("/a", async(ctx) =>[|{
        //a
    }|])|]
    .get[|("/", async(ctx) =>[|{
        ctx.body = "base";
    }|])|]
    .post[|("/a", async(ctx) =>[|{
        //a
    }|])|]
    .get[|("/", async(ctx) =>[|{
        ctx.body = "base";
    }|])|]
    .post[|("/a", async(ctx) =>[|{
        //a
    }|])|]
    .get[|("/", async(ctx) =>[|{
        ctx.body = "base";
    }|])|]
    .post[|("/a", async(ctx) =>[|{
        //a
    }|])|]
    .get[|("/", async(ctx) =>[|{
        ctx.body = "base";
    }|])|]
    .post[|("/a", async(ctx) =>[|{
        //a
    }|])|]
    .get[|("/", async(ctx) =>[|{
        ctx.body = "base";
    }|])|]
    .post[|("/a", async(ctx) =>[|{
        //a
    }|])|]
    .get[|("/", async(ctx) =>[|{
        ctx.body = "base";
    }|])|]
    .post[|("/a", async(ctx) =>[|{
        //a
    }|])|]
    .get[|("/", async(ctx) =>[|{
        ctx.body = "base";
    }|])|]
    .post[|("/a", async(ctx) =>[|{
        //a
    }|])|]
    .get[|("/", async(ctx) =>[|{
        ctx.body = "base";
    }|])|]
    .post[|("/a", async(ctx) =>[|{
        //a
    }|])|]
    .get[|("/", async(ctx) =>[|{
        ctx.body = "base";
    }|])|]
    .post[|("/a", async(ctx) =>[|{
        //a
    }|])|]
    .get[|("/", async(ctx) =>[|{
        ctx.body = "base";
    }|])|]
    .post[|("/a", async(ctx) =>[|{
        //a
    }|])|]
    .get[|("/", async(ctx) =>[|{
        ctx.body = "base";
    }|])|]
    .post[|("/a", async(ctx) =>[|{
        //a
    }|])|]
    .get[|("/", async(ctx) =>[|{
        ctx.body = "base";
    }|])|]
    .post[|("/a", async(ctx) =>[|{
        //a
    }|])|]
    .get[|("/", async(ctx) =>[|{
        ctx.body = "base";
    }|])|]
    .post[|("/a", async(ctx) =>[|{
        //a
    }|])|]
    .get[|("/", async(ctx) =>[|{
        ctx.body = "base";
    }|])|]
    .post[|("/a", async(ctx) =>[|{
        //a
    }|])|]
    .get[|("/", async(ctx) =>[|{
        ctx.body = "base";
    }|])|]
    .post[|("/a", async(ctx) =>[|{
        //a
    }|])|]
    .get[|("/", async(ctx) =>[|{
        ctx.body = "base";
    }|])|]
    .post[|("/a", async(ctx) =>[|{
        //a
    }|])|]
    .get[|("/", async(ctx) =>[|{
        ctx.body = "base";
    }|])|]
    .post[|("/a", async(ctx) =>[|{
        //a
    }|])|]"#;
    let mut s = Session::new_for_test("getOutliningSpansDepthChainedCalls", content);
    fourslash::unsupported("VerifyOutliningSpans"); // f.VerifyOutliningSpans(t)
}
