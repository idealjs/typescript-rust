pub fn stringify_args(args: &[String]) -> Vec<String> { crate::fntrace::enter("stringify_args"); 
    if args.is_empty() {
        return Vec::new();
    }
    args.to_vec()
}
