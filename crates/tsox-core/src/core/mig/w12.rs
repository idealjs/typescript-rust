use super::super::tristate::Tristate;

impl Tristate {
    pub fn unmarshal_json(data: &[u8]) -> Tristate {
        match std::str::from_utf8(data).unwrap_or("") {
            "true" => Tristate::True,
            "false" => Tristate::False,
            _ => Tristate::Unknown,
        }
    }
}
