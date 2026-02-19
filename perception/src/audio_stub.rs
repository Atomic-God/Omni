use crate::traits::{ModalEncoder, ModalDecoder};
use core_vsa::HyperVector;

pub struct AudioStub;

impl ModalEncoder for AudioStub {
    fn encode_bytes(&self, _data: &[u8]) -> HyperVector {
        HyperVector::random()
    }
}

impl ModalDecoder for AudioStub {
    fn decode_bytes(&self, _hv: &HyperVector) -> Vec<u8> {
        Vec::new()
    }
}
