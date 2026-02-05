use core_vsa::HyperVector;

pub trait Encoder {
    fn encode(&self, input: &str) -> HyperVector;
}

pub trait Decoder {
    fn decode(&self, hv: &HyperVector) -> String;
}

pub trait ModalEncoder {
    fn encode_bytes(&self, data: &[u8]) -> HyperVector;
}
