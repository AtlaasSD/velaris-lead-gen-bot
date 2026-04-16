mod models;
mod error;
mod config;
mod scraper;
mod validator;
mod reporter;
mod persistence;

use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};
use std::time::Instant;

// ─── ASCII Art + Banner ───────────────────────────────────────────────────────

fn print_banner() {
    println!("{}", "
  ██╗   ██╗███████╗██╗      █████╗ ██████╗ ██╗███████╗
  ██║   ██║██╔════╝██║     ██╔══██╗██╔══██╗██║██╔════╝
  ██║   ██║█████╗  ██║     ███████║██████╔╝██║███████╗
  ╚██╗ ██╔╝██╔══╝  ██║     ██╔══██║██╔══██╗██║╚════██║
   ╚████╔╝ ███████╗███████╗██║  ██║██║  ██║██║███████║
    ╚═══╝  ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝╚══════╝".cyan().bold());
    println!("{}", "                     L E A D - G E N   B O T".cyan());
    println!("{}", "  ─────────────────────────────────────────────".dimmed());
    println!("{}", "  Desarrollado por Velaris Agency  |  v1.0.0".dimmed());
    println!("{}", "  ─────────────────────────────────────────────\n".dimmed());
}

fn print_step(step: u8, total: u8, title: &str) {
    println!(
        "\n  {} {} {}",
        format!("[{}/{}]", step, total).cyan().bold(),
        "▶".white().bold(),
        title.white().bold()
    );
    println!("  {}", "─".repeat(50).dimmed());
}

fn print_success(msg: &str) {
    println!("  {} {}", "✔".green().bold(), msg.green());
}

fn print_result_box(title: &str, lines: &[(&str, String)]) {
    println!("\n  ╔{}╗", "═".repeat(54).cyan().to_string());
    println!("  ║  {:<52} ║", title.bold());
    println!("  ╠{}╣", "═".repeat(54).cyan().to_string());
    for (label, value) in lines {
        println!("  ║  {:<28}{:>24} ║", label.dimmed(), value.yellow().bold());
    }
    println!("  ╚{}╝\n", "═".repeat(54).cyan().to_string());
}

// ─── Selección de categoría ───────────────────────────────────────────────────

fn select_category() -> String {
    println!("  {} Elige el tipo de negocio a buscar:\n", "◆".cyan());
    let options = [
        ("1", "Restaurantes"),
        ("2", "Cafés"),
        ("3", "Bares"),
        ("4", "Panaderías"),
        ("5", "Peluquerías"),
        ("6", "Clínicas / Consultorios"),
        ("7", "Talleres mecánicos"),
        ("8", "Farmacias"),
        ("9", "Hoteles"),
        ("0", "Otro (digitar manualmente)"),
    ];

    for (key, label) in &options {
        let k = format!("  [{}]", key).cyan().bold();
        println!("  {}  {}", k, label);
    }
    println!();

    loop {
        print!("  {} Opción: ", "→".cyan());
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let choice = input.trim();

        let cat = match choice {
            "1" => Some("Restaurantes".to_string()),
            "2" => Some("Cafés".to_string()),
            "3" => Some("Bares".to_string()),
            "4" => Some("Panaderías".to_string()),
            "5" => Some("Peluquerías".to_string()),
            "6" => Some("Clínicas".to_string()),
            "7" => Some("Talleres mecánicos".to_string()),
            "8" => Some("Farmacias".to_string()),
            "9" => Some("Hoteles".to_string()),
            "0" => {
                print!("  {} Escribe el tipo de negocio: ", "→".cyan());
                io::stdout().flush().ok();
                let mut custom = String::new();
                io::stdin().read_line(&mut custom).ok();
                let trimmed = custom.trim().to_string();
                if trimmed.is_empty() { None } else { Some(trimmed) }
            }
            _ => None,
        };

        if let Some(c) = cat {
            println!("  {} Categoría seleccionada: {}\n", "✔".green(), c.yellow().bold());
            return c;
        }
        println!("  {} Opción no válida. Intenta de nuevo.", "✗".red());
    }
}

// ─── Lectura de input con prompt bonito ───────────────────────────────────────

fn read_line_prompt(prompt: &str) -> String {
    print!("  {} {}: ", "→".cyan(), prompt.bold());
    io::stdout().flush().ok();
    let mut s = String::new();
    io::stdin().read_line(&mut s).ok();
    s.trim().to_string()
}

// ─── Main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    // Limpiar pantalla y mostrar banner
    print!("\x1B[2J\x1B[H"); // ANSI clear screen
    print_banner();

    // ═══════════════════════════════════════════════════════
    // PASO 1 — CONFIGURACIÓN DE BÚSQUEDA
    // ═══════════════════════════════════════════════════════
    print_step(1, 4, "CONFIGURACIÓN DE BÚSQUEDA");

    let zona = loop {
        let z = read_line_prompt("Zona (ej. Pereira, Risaralda)");
        if !z.is_empty() { break z; }
        println!("  {} La zona no puede estar vacía.", "✗".red());
    };

    println!();
    let categoria = select_category();

    let min_reviews: u32 = loop {
        let r = read_line_prompt("Mínimo de reseñas (ej. 10, Enter para 0)");
        if r.is_empty() { break 0; }
        match r.parse::<u32>() {
            Ok(n) => break n,
            Err(_) => println!("  {} Ingresa un número válido.", "✗".red()),
        }
    };

    let min_score: f32 = loop {
        let r = read_line_prompt("Calificación mínima (ej. 4.0, Enter para 0.0)");
        if r.is_empty() { break 0.0; }
        match r.replace(',', ".").parse::<f32>() {
            Ok(n) => break n,
            Err(_) => println!("  {} Ingresa un número válido (usa punto o coma decimal).", "✗".red()),
        }
    };

    // Confirmar configuración
    print_result_box("CONFIGURACIÓN CONFIRMADA", &[
        ("Zona", zona.clone()),
        ("Categoría", categoria.clone()),
        ("Reseñas mínimas", min_reviews.to_string()),
        ("Calificación mínima", format!("{:.1} ★", min_score)),
    ]);

    // ═══════════════════════════════════════════════════════
    // PASO 2 — INICIALIZACIÓN DEL BROWSER
    // ═══════════════════════════════════════════════════════
    print_step(2, 4, "INICIALIZANDO NAVEGADOR");

    println!("  {} Cargando Microsoft Edge en modo headless...", "◆".cyan());
    let session_start = Instant::now();

    // Cargar sesión previa si coincide con zona + categoría
    let (session, mut all_leads) = match persistence::load_session()? {
        Some((s, l)) if s.zona == zona && s.categoria == categoria => {
            println!("  {} Sesión previa cargada: {} leads acumulados.", "✔".green(), l.len().to_string().yellow().bold());
            (s, l)
        }
        _ => {
            println!("  {} Nueva sesión iniciada.", "◆".cyan());
            (
                models::SearchSession {
                    zona: zona.clone(),
                    categoria: categoria.clone(),
                    ultima_actualizacion: chrono::Utc::now(),
                },
                vec![],
            )
        }
    };

    let (mut browser, _handler) = scraper::init_browser().await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let page = browser.new_page("https://www.google.com/maps").await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    print_success("Navegador listo.");

    // ═══════════════════════════════════════════════════════
    // PASO 3 — SCRAPING
    // ═══════════════════════════════════════════════════════
    print_step(3, 4, "SCRAPING EN GOOGLE MAPS");

    let mut new_leads = scraper::scrape_maps(
        &page, &session.categoria, &session.zona, min_reviews, min_score,
    ).await.map_err(|e| anyhow::anyhow!("{}", e))?;

    // ── Validación de webs
    validator::validate_leads(&mut new_leads).await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // ── Unificar, deduplicar y filtrar
    let pre_count = all_leads.len();
    all_leads.extend(new_leads.clone());

    let mut seen_urls = std::collections::HashSet::new();
    all_leads.retain(|l| {
        let is_unique = seen_urls.insert(l.maps_url.clone());
        let needs_web = l.estado_web != models::WebStatus::Activa;
        is_unique && needs_web
    });

    let post_count  = all_leads.len();
    let added_count = post_count.saturating_sub(pre_count);

    // Guardar sesión actualizada
    persistence::save_session(&session, &all_leads)?;

    let elapsed = session_start.elapsed();

    // Resumen de resultados
    print_result_box("RESUMEN DE EXTRACCIÓN", &[
        ("Leads extraídos esta sesión",  new_leads.len().to_string()),
        ("Leads únicos añadidos",         added_count.to_string()),
        ("Total acumulado en base",        post_count.to_string()),
        ("Con teléfono",                   new_leads.iter().filter(|l| l.telefono.is_some()).count().to_string()),
        ("Tiempo total",                   format!("{:.1}s", elapsed.as_secs_f32())),
    ]);

    // ═══════════════════════════════════════════════════════
    // PASO 4 — GENERACIÓN DEL PDF
    // ═══════════════════════════════════════════════════════
    print_step(4, 4, "GENERANDO REPORTE PDF");

    if all_leads.is_empty() {
        println!("  {} No se encontraron leads calificados para esta búsqueda.", "⚠".yellow().bold());
        println!("  {} Intenta con menos filtros o una zona diferente.\n", "◆".cyan());
    } else {
        let city_clean = zona.split(',').next().unwrap_or(&zona).trim()
            .to_lowercase().replace(' ', "_");
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M").to_string();
        let output_name = format!("leads_{}_{}.pdf", city_clean, timestamp);
        let output_path = std::env::current_dir()
            .unwrap_or_default()
            .join(&output_name);

        println!("  {} Compilando {} filas en el reporte...", "◆".cyan(), all_leads.len());

        reporter::generate_report(&page, &all_leads, &output_name, &zona).await
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        print_success(&format!("PDF generado exitosamente."));
        println!("\n  {} {}", "📄 Ruta:".bold(), output_path.display().to_string().cyan().underline());
        println!();
    }

    browser.close().await.ok();

    println!("{}", "\n  ─────────────────────────────────────────────".dimmed());
    println!("  {} Sesión finalizada. ¡Hasta la próxima!", "◈".green().bold());
    println!("{}\n", "  ─────────────────────────────────────────────".dimmed());

    Ok(())
}
