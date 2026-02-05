use cognition::CognitionCore;
use std::fs::File;
use std::io::BufReader;

pub fn save(core: &CognitionCore, path: &str) -> Result<(), std::io::Error> {
    let file = File::create(path)?;
    serde_json::to_writer(file, core)?;
    Ok(())
}

pub fn load(path: &str) -> Result<CognitionCore, std::io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let core = serde_json::from_reader(reader)?;
    Ok(core)
}
