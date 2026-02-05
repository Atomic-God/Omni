use crate::traits::{ModalEncoder, ModalDecoder};
use core_vsa::HyperVector;

pub struct VisionStub;

impl ModalEncoder for VisionStub {
    fn encode_bytes(&self, _data: &[u8]) -> HyperVector {
        // Stub: returns random HV for now
        HyperVector::random()
    }
}

impl ModalDecoder for VisionStub {
    fn decode_bytes(&self, _hv: &HyperVector) -> Vec<u8> {
        Vec::new()
    }
}
