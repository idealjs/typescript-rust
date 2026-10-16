use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("getOutliningSpansDepthChainedCalls", content);
    // TODO: f.VerifyOutliningSpans(t)
}
