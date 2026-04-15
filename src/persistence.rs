use crate::models::{Lead, SearchSession};
use crate::error::Result;
use crate::config::PERSISTENCE_FILE;
use std::fs::File;
use std::io::{BufReader, BufWriter};

/// Guarda la sesión actual y los leads en disco (sobreescribe)
pub fn save_session(session: &SearchSession, leads: &[Lead]) -> Result<()> {
    let file = File::create(PERSISTENCE_FILE)
        .map_err(|e| crate::error::BotError::PersistenceError(e.to_string()))?;

    serde_json::to_writer_pretty(BufWriter::new(file), &(session, leads))
        .map_err(|e| crate::error::BotError::PersistenceError(e.to_string()))?;

    Ok(())
}

/// Carga la sesión y leads previos, si existen
pub fn load_session() -> Result<Option<(SearchSession, Vec<Lead>)>> {
    if !std::path::Path::new(PERSISTENCE_FILE).exists() {
        return Ok(None);
    }

    let file = File::open(PERSISTENCE_FILE)
        .map_err(|e| crate::error::BotError::PersistenceError(e.to_string()))?;

    let data: (SearchSession, Vec<Lead>) = serde_json::from_reader(BufReader::new(file))
        .map_err(|e| crate::error::BotError::PersistenceError(e.to_string()))?;

    Ok(Some(data))
}
