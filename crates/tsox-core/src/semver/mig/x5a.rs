pub(crate) fn get_uint_component(text: &str) -> Result<u32, std::num::ParseIntError> {
    text.parse::<u32>()
}
