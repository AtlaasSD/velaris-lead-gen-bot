use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::handler::viewport::Viewport;
use futures::StreamExt;
use crate::error::Result;
use crate::config::DEFAULT_USER_AGENT;

pub async fn init_browser() -> Result<(Browser, tokio::task::JoinHandle<()>)> {
    // Directorio de perfil dinámico para aislar cada ejecución y evitar ExitStatus(21)
    let unique_id = uuid::Uuid::new_v4().to_string();
    let temp_profile = std::env::temp_dir().join(format!("velaris_browser_{}", unique_id));

    let config = BrowserConfig::builder()
        .chrome_executable("C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe")
        .window_size(1920, 1080)
        .viewport(Viewport {
            width: 1920,
            height: 1080,
            ..Default::default()
        })
        .arg("--headless=new")
        .arg(format!("--user-data-dir={}", temp_profile.display()))
        .arg(format!("--user-agent={}", DEFAULT_USER_AGENT))
        .arg("--disable-blink-features=AutomationControlled")
        .arg("--no-sandbox")
        .arg("--disable-gpu")
        .arg("--disable-dev-shm-usage")
        .arg("--no-first-run")
        .build()
        .map_err(|e| crate::error::BotError::BrowserError(e))?;

    let (browser, mut handler) = Browser::launch(config).await
        .map_err(|e| crate::error::BotError::BrowserError(e.to_string()))?;

    // Debemos correr el handler en una tarea separada para que la comunicacion CDP fluya
    let handle = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if let Err(e) = h {
                let err_msg = format!("{:?}", e);
                // Ignoramos errores de parseo Serde (comunes en discrepancias de version CDP)
                if !err_msg.contains("Serde") && !err_msg.contains("data did not match") {
                    eprintln!("Browser handler error: {}", err_msg);
                }
            }
        }
    });

    Ok((browser, handle))
}
