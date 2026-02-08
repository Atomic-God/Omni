use crate::MindPack;
use std::io::{Read, Write};

pub trait MindSerializer {
    fn serialize<W: Write>(&self, mind: &MindPack, writer: W) -> Result<(), std::io::Error>;
}

pub trait MindDeserializer {
    fn deserialize<R: Read>(&self, reader: R) -> Result<MindPack, std::io::Error>;
}

pub struct StandardSerializer;

impl MindSerializer for StandardSerializer {
    fn serialize<W: Write>(&self, mind: &MindPack, mut writer: W) -> Result<(), std::io::Error> {
        let bin_config = bincode::config::standard();
        bincode::serde::encode_into_std_write(mind, &mut writer, bin_config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        Ok(())
    }
}

impl MindDeserializer for StandardSerializer {
    fn deserialize<R: Read>(&self, mut reader: R) -> Result<MindPack, std::io::Error> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        let bin_config = bincode::config::standard();
        let (pack, _) : (MindPack, usize) = bincode::serde::decode_from_slice(&buffer, bin_config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        Ok(pack)
    }
}
