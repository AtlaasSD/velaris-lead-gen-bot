use thiserror::Error;

#[derive(Error, Debug)]
pub enum BotError {
    #[error("Error en el navegador: {0}")]
    BrowserError(String),

    #[error("Error de red: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Error de persistencia: {0}")]
    PersistenceError(String),

    #[error("Error al generar PDF: {0}")]
    PdfError(String),

    #[allow(dead_code)]
    #[error("Error desconocido: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, BotError>;
