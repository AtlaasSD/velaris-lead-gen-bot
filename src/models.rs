use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Estado de validación del sitio web de un lead
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WebStatus {
    /// El negocio no tiene sitio web propio (o solo tiene redes sociales)
    SinWeb,
    /// El sitio respondió correctamente
    Activa,
    /// El sitio devolvió un código de error HTTP
    ErrorServidor(u16),
    /// La conexión excedió el tiempo de espera
    Timeout,
    /// El sitio no responde o está caído
    Caida(String),
}

/// Lead calificado listo para el reporte
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lead {
    pub id: String,
    pub nombre: String,
    pub direccion: Option<String>,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub categoria: String,
    pub url_web: Option<String>,
    pub maps_url: String,
    pub estado_web: WebStatus,
    pub reseñas: u32,
    pub calificacion: f32,
    pub extraido_el: DateTime<Utc>,
}

/// Sesión de búsqueda para persistencia entre ejecuciones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSession {
    pub zona: String,
    pub categoria: String,
    pub ultima_actualizacion: DateTime<Utc>,
}
