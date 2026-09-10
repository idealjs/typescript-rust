use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn formatting_jsx_texts1() {
    let content = r#"//@Filename: file.tsx
<option>
    homu   ;      homu
    homu;homu
    homu   :    homu
    homu:homu
    homu    ?     homu
    homu    .    homu

    homu    [   homu   ]   homu

    !     homu
    --    Type
    homu    --
    homu    ++
    ++     homu

    homu  ,   homu

    var    homu
    throw    homu
    new    homu
    delete   homu
    return       homu
    typeof     homu
    await     homu

    abstract  homu
    class     homu
    declare   homu
    default   homu
    enum      homu
    export    homu
    homu    extends   homu
    get       homu
    homu    implements     homu
    interface      homu
    module    homu
    namespace      homu
    private   homu
    public    homu
    protected      homu
    set       homu
    static    homu
    type      homu

    homu    =>    homu
    homu=>homu

    ...       homu

    homu     @     homu
    homu@homu

    (    homu   )    homu
</option>;"#;
    let mut s = Session::new_for_test("formattingJsxTexts1", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"<option>
    homu   ;      homu
    homu;homu
    homu   :    homu
    homu:homu
    homu    ?     homu
    homu    .    homu

    homu    [   homu   ]   homu

    !     homu
    --    Type
    homu    --
    homu    ++
    ++     homu

    homu  ,   homu

    var    homu
    throw    homu
    new    homu
    delete   homu
    return       homu
    typeof     homu
    await     homu

    abstract  homu
    class     homu
    declare   homu
    default   homu
    enum      homu
    export    homu
    homu    extends   homu
    get       homu
    homu    implements     homu
    interface      homu
    module    homu
    namespace      homu
    private   homu
    public    homu
    protected      homu
    set       homu
    static    homu
    type      homu

    homu    =>    homu
    homu=>homu

    ...       homu

    homu     @     homu
    homu@homu

    (    homu   )    homu
</option>;"#);
}
