use core_vsa::HyperVector;

pub trait PerceptionModule {
    fn learn_text(&mut self, text: &str);
}

pub trait MemoryModule {
    fn save(&self, path: &str) -> Result<(), std::io::Error>;
    fn load(path: &str) -> Result<Self, std::io::Error> where Self: Sized;
}

pub trait ReasoningModule {
    fn infer(&self, start: &str, target: &str) -> bool;
    fn query(&self, query_str: &str) -> String;
    fn sentence_vector(&self, sentence: &str) -> Option<HyperVector>;
}
