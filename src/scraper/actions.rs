use chromiumoxide::Page;
use crate::error::Result;
use crate::models::Lead;
use std::time::Duration;
use tokio::time::sleep;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

// ─── Blacklist de dominios que NO cuentan como sitio web corporativo ──────────
const SOCIAL_BLACKLIST: &[&str] = &[
    "facebook.com",
    "instagram.com",
    "wa.me",
    "whatsapp.com",
    "api.whatsapp.com",
    "linktr.ee",
    "linktree.com",
    "drive.google.com",
    "docs.google.com",
    "tiktok.com",
    "twitter.com",
    "x.com",
    "youtube.com",
    "t.me",
    "telegram.me",
];

fn is_social_or_blacklisted(url: &str) -> bool {
    let lower = url.to_lowercase();
    SOCIAL_BLACKLIST.iter().any(|&p| lower.contains(p))
}

// ─── Helpers de output coloreado ─────────────────────────────────────────────

fn log_info(msg: &str) {
    println!("{} {}", "  ◆".cyan().bold(), msg);
}

fn log_ok(msg: &str) {
    println!("{} {}", "  ✓".green().bold(), msg.green());
}

fn log_skip(msg: &str) {
    println!("{} {}", "  ✗".red().bold(), msg.dimmed());
}

fn log_warn(msg: &str) {
    println!("{} {}", "  ~".yellow().bold(), msg.yellow());
}

fn log_section(msg: &str) {
    println!("\n{}", format!("  ┌─  {}  ─┐", msg).cyan().bold());
}

// ─── Función principal de scraping ───────────────────────────────────────────

/// Navega Google Maps, hace scroll continuo y extrae leads sin web propia
pub async fn scrape_maps(
    page: &Page,
    categoria: &str,
    zona: &str,
    min_reviews: u32,
    min_score: f32,
) -> Result<Vec<Lead>> {
    // 1 ── Navegación inicial
    let search_url = format!("https://www.google.com/maps/search/{}+en+{}", categoria, zona);
    log_info(&format!("Navegando a Google Maps → {}", search_url.dimmed()));

    page.goto(&search_url).await
        .map_err(|e| crate::error::BotError::BrowserError(e.to_string()))?;

    log_info("Esperando que cargue el feed de resultados...");
    sleep(Duration::from_secs(5)).await;

    log_section("MINERÍA EN CURSO");

    // 2 ── Estado del scraper
    let mut all_leads_map: std::collections::HashMap<String, Lead> = std::collections::HashMap::new();
    let mut processed_urls: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut stagnant_scrolls = 0u32;
    let mut total_analyzed = 0u32;
    const MAX_STAGNANT: u32 = 10;
    const MAX_BATCHES: u32 = 120;

    // Barra de progreso de estancamiento (scroll hasta fin)
    let scroll_bar = ProgressBar::new(MAX_STAGNANT as u64);
    scroll_bar.set_style(
        ProgressStyle::with_template(
            "  {spinner:.cyan} Avance scroll [{bar:30.cyan/blue}] {pos}/{len} zonas sin novedades"
        )
        .unwrap()
        .progress_chars("█▓░")
    );

    // 3 ── Bucle principal de scraping
    for batch_id in 0..MAX_BATCHES {
        // ── DESCUBRIMIENTO ────────────────────────────────────────────────
        #[derive(serde::Deserialize, Debug)]
        struct RawCard {
            nombre: String,
            maps_url: String,
            reseñas: String,
            calificacion: String,
        }

        let candidates: Vec<RawCard> = page.evaluate(format!(r#"
            () => {{
                const results = [];
                const cards = [...document.querySelectorAll('a[href*="/maps/place/"]')];
                for (let card of cards) {{
                    if (card.dataset.velarisProcessed) continue;
                    const mapsUrl = card.href;
                    const nameNode = card.getAttribute('aria-label');
                    const parent = card.closest('[role="article"]') || card.parentElement;
                    let ratingText = "0", reviewsText = "0";
                    if (parent) {{
                        const t = parent.innerText;
                        const m = t.match(/(\d[\.,]\d)\s*\(([\d\.]+)\)/);
                        if (m) {{ ratingText = m[1]; reviewsText = m[2]; }}
                    }}
                    const revs = parseInt(reviewsText.replace(/\D/g, '')) || 0;
                    const sc   = parseFloat(ratingText.replace(',', '.')) || 0.0;
                    if (revs >= {} && sc >= {}) {{
                        results.push({{ nombre: nameNode || "Sin nombre", maps_url: mapsUrl, reseñas: reviewsText, calificacion: ratingText }});
                    }} else {{
                        card.dataset.velarisProcessed = "skip";
                    }}
                }}
                return results;
            }}
        "#, min_reviews, min_score)).await
            .map_err(|e| crate::error::BotError::BrowserError(e.to_string()))?
            .into_value()
            .unwrap_or_default();

        let new_candidates: Vec<RawCard> = candidates
            .into_iter()
            .filter(|c| !processed_urls.contains(&c.maps_url))
            .collect();

        // ── SIN CANDIDATOS → SCROLL ───────────────────────────────────────
        if new_candidates.is_empty() {
            page.evaluate(r#"() => {
                const f = document.querySelector('div[role="feed"]');
                if (f) f.scrollBy(0, 1500);
            }"#).await.ok();
            sleep(Duration::from_millis(1800)).await;

            stagnant_scrolls += 1;
            scroll_bar.set_position(stagnant_scrolls as u64);

            if stagnant_scrolls >= MAX_STAGNANT {
                scroll_bar.finish_with_message("Fin del mapa alcanzado");
                println!("\n  {} {}", "◈".cyan().bold(), "Scraping completado — no hay más negocios por analizar.".cyan());
                break;
            }
            continue;
        }

        // ── CON CANDIDATOS → PROCESAR UNO A UNO ──────────────────────────
        stagnant_scrolls = 0;
        scroll_bar.set_position(0);
        println!(
            "\n  {} {} candidatos nuevos encontrados en el batch {}",
            "◈".cyan().bold(),
            new_candidates.len().to_string().yellow().bold(),
            batch_id
        );

        for raw in new_candidates {
            processed_urls.insert(raw.maps_url.clone());
            total_analyzed += 1;

            println!(
                "\n  {} {} {}",
                format!("[{}]", total_analyzed).dimmed(),
                "Analizando:".bold(),
                raw.nombre.white().bold()
            );

            // ── CLIC ──────────────────────────────────────────────────────
            let maps_url_escaped = raw.maps_url.replace('"', "\\\"");
            let clicked: bool = page.evaluate(format!(r#"
                () => {{
                    const card = document.querySelector('a[href="{}"]');
                    if (card) {{
                        card.dataset.velarisProcessed = "done";
                        card.scrollIntoView({{behavior: 'instant', block: 'center'}});
                        card.click();
                        return true;
                    }}
                    return false;
                }}
            "#, maps_url_escaped)).await
                .map(|r| r.into_value().unwrap_or(false))
                .unwrap_or(false);

            if !clicked {
                log_skip("No se encontró la tarjeta en el DOM. Saltando.");
                continue;
            }

            log_info("Abriendo panel de detalles...");
            sleep(Duration::from_millis(2500)).await;

            // ── EXTRACCIÓN ────────────────────────────────────────────────
            #[derive(serde::Deserialize)]
            struct Detail {
                phone: Option<String>,
                web: Option<String>,
                addr: Option<String>,
            }

            let detail: Detail = page.evaluate(r#"
                () => {
                    let p = null, w = null, a = null;

                    // Estrategia 1: data-item-id estándar
                    const phoneNode = document.querySelector('button[data-item-id^="phone:tel"]');
                    if (phoneNode) p = phoneNode.getAttribute('aria-label') || phoneNode.innerText;

                    // Estrategia 2: aria-label con palabras clave
                    if (!p) {
                        const pb = document.querySelector(
                            'button[aria-label*="Teléfono"], button[aria-label*="telefono"], button[aria-label*="Llamar"], button[aria-label*="Phone"], button[aria-label*="Call"]'
                        );
                        if (pb) p = pb.getAttribute('aria-label') || pb.innerText;
                    }

                    // Estrategia 3: enlace tel:
                    if (!p) {
                        const tl = document.querySelector('a[href^="tel:"]');
                        if (tl) p = tl.href.replace('tel:', '').trim();
                    }

                    // Estrategia 4: regex en el texto del panel
                    if (!p) {
                        const panel = document.querySelector('[role="main"]') || document.body;
                        const m = (panel ? panel.innerText : document.body.innerText)
                            .match(/(\+57[\s\-]?)?(\(?\d{1,3}\)?[\s\-]?)?(3\d{2}[\s\-]?\d{3}[\s\-]?\d{4}|6\d{7,9}|60\d[\s\-]?\d{7})/);
                        if (m) p = m[0];
                    }

                    if (p) p = p.replace(/Teléfono:|Telefono:|Llamar:|Phone:|Call:/ig, '').replace(/\s+/g, ' ').trim();

                    const wn = document.querySelector('a[data-item-id="authority"]');
                    if (wn) w = wn.href;

                    const an = document.querySelector('button[data-item-id="address"]');
                    if (an) a = (an.getAttribute('aria-label') || an.innerText).replace(/Dirección:|Address:/ig, '').trim();

                    return { phone: p, web: w, addr: a };
                }
            "#).await
                .ok()
                .and_then(|ev| ev.into_value::<serde_json::Value>().ok())
                .and_then(|v| serde_json::from_value::<Detail>(v).ok())
                .unwrap_or(Detail { phone: None, web: None, addr: None });

            // Log del teléfono extraído
            match &detail.phone {
                Some(tel) => log_ok(&format!("Teléfono extraído: {}", tel.bold())),
                None      => log_warn("Sin teléfono disponible en el panel"),
            }

            // ── VOLVER A LA LISTA ─────────────────────────────────────────
            page.evaluate(r#"
                () => {
                    const b = document.querySelector('button[aria-label*="Atrás"]')
                           || document.querySelector('button[aria-label*="Back"]')
                           || document.querySelector('button[jsaction*="pane.back"]')
                           || document.querySelector('.VfPpkd-icon-LgbsSe');
                    if (b) b.click();
                }
            "#).await.ok();
            sleep(Duration::from_millis(1000)).await;

            // ── FILTRO WEB ────────────────────────────────────────────────
            if let Some(ref url) = detail.web {
                if is_social_or_blacklisted(url.as_str()) {
                    log_warn(&format!("Web en blacklist ({}). Incluido como sin web.", url));
                } else {
                    log_skip(&format!("Descartado — tiene web corporativa: {}", url));
                    continue;
                }
            } else {
                log_info("Sin sitio web detectado → candidato válido");
            }

            // ── CONSTRUIR LEAD ────────────────────────────────────────────
            let rec_reviews = raw.reseñas
                .chars().filter(|c| c.is_ascii_digit())
                .collect::<String>().parse::<u32>().unwrap_or(0);

            let rec_score = raw.calificacion
                .replace(',', ".").split_whitespace().next()
                .and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);

            log_ok(&format!(
                "Lead capturado → {} | ☎ {} | ★ {:.1} ({} reseñas)",
                raw.nombre.bold(),
                detail.phone.as_deref().unwrap_or("N/A").bold(),
                rec_score,
                rec_reviews
            ));

            all_leads_map.insert(raw.maps_url.clone(), Lead {
                id: uuid::Uuid::new_v4().to_string(),
                nombre: raw.nombre,
                direccion: detail.addr,
                telefono: detail.phone,
                correo: None,
                categoria: categoria.to_string(),
                url_web: detail.web,
                maps_url: raw.maps_url,
                estado_web: crate::models::WebStatus::SinWeb,
                reseñas: rec_reviews,
                calificacion: rec_score,
                extraido_el: chrono::Utc::now(),
            });
        }

        // ── SCROLL AL FINAL DEL BATCH ─────────────────────────────────────
        page.evaluate(r#"() => {
            const f = document.querySelector('div[role="feed"]');
            if (f) f.scrollBy(0, 2500);
        }"#).await.ok();
        sleep(Duration::from_millis(1800)).await;
    }

    let final_leads: Vec<Lead> = all_leads_map.into_values().collect();
    println!(
        "\n  {} Minería completada — {} leads calificados extraídos\n",
        "◈".green().bold(),
        final_leads.len().to_string().green().bold()
    );
    Ok(final_leads)
}
