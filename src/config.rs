/// User-Agent para el navegador y las peticiones HTTP
pub const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Timeout en segundos para verificar si una web responde
pub const WEB_CHECK_TIMEOUT_SECS: u64 = 5;

/// Archivo de sesión para persistencia entre ejecuciones
pub const PERSISTENCE_FILE: &str = "session_leads.json";
