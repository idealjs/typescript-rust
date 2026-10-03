pub(crate) fn get_uint_component(text: &str) -> Result<u32, std::num::ParseIntError> { crate::fntrace::enter("get_uint_component"); 
    text.parse::<u32>()
}
