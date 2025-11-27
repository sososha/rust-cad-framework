use cad_core::Entity;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IOError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization Error: {0}")]
    Serialization(String),
}

pub trait CADSerializer {
    fn save(&self, entities: &[Entity], path: &Path) -> Result<(), IOError>;
    fn load(&self, path: &Path) -> Result<Vec<Entity>, IOError>;
}

pub struct JSONSerializer;

impl CADSerializer for JSONSerializer {
    fn save(&self, entities: &[Entity], path: &Path) -> Result<(), IOError> {
        let file = std::fs::File::create(path)?;
        serde_json::to_writer_pretty(file, entities)
            .map_err(|e| IOError::Serialization(e.to_string()))?;
        Ok(())
    }

    fn load(&self, path: &Path) -> Result<Vec<Entity>, IOError> {
        let file = std::fs::File::open(path)?;
        let entities: Vec<Entity> = serde_json::from_reader(file)
            .map_err(|e| IOError::Serialization(e.to_string()))?;
        Ok(entities)
    }
}
