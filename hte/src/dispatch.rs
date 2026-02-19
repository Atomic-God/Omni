pub trait SimdBackend {
    fn xor(&self, a: &[u8], b: &[u8]) -> Vec<u8>;
    fn bind(&self, a: &[u8], b: &[u8]) -> Vec<u8>;
    fn bundle(&self, a: &[i32], b: &[i32]) -> Vec<i32>;
}

pub struct ScalarBackend;

impl SimdBackend for ScalarBackend {
    fn xor(&self, a: &[u8], b: &[u8]) -> Vec<u8> {
        a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
    }

    fn bind(&self, a: &[u8], b: &[u8]) -> Vec<u8> {
        self.xor(a, b) // Binding in VSA is often XOR
    }

    fn bundle(&self, a: &[i32], b: &[i32]) -> Vec<i32> {
        a.iter().zip(b.iter()).map(|(x, y)| x + y).collect()
    }
}
