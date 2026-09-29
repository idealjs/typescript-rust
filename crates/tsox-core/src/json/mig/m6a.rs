use std::io::{Read, Write};

pub fn marshal_encode<W: Write, T: serde::Serialize>(
    out: &mut serde_json::Serializer<W>,
    value: &T,
) -> Result<(), serde_json::Error> {
    value.serialize(out)
}

pub fn unmarshal_decode<'de, R: Read, T: serde::Deserialize<'de>>(
    decoder: &mut serde_json::Deserializer<serde_json::de::IoRead<R>>,
) -> Result<T, serde_json::Error> {
    T::deserialize(decoder)
}

pub fn unmarshal_read<R: Read, T: serde::de::DeserializeOwned>(
    input: R,
) -> Result<T, serde_json::Error> {
    T::deserialize(&mut serde_json::Deserializer::from_reader(input))
}

pub fn new_decoder<R: Read>(r: R) -> serde_json::Deserializer<serde_json::de::IoRead<R>> {
    serde_json::Deserializer::from_reader(r)
}
