use crate::models::{Lead, WebStatus};
use crate::error::Result;
use crate::config::WEB_CHECK_TIMEOUT_SECS;
use reqwest::Client;
use std::time::Duration;
use futures::future::join_all;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

/// Dominios que NO cuentan como sitio web empresarial
fn is_social_media(url: &str) -> bool {
    const BLACKLIST: &[&str] = &[
        "facebook.com", "instagram.com", "wa.me", "whatsapp.com",
        "api.whatsapp.com", "tiktok.com", "twitter.com", "x.com",
        "youtube.com", "t.me", "telegram.me", "linktr.ee", "linktree.com",
        "drive.google.com", "docs.google.com", "dropbox.com",
    ];
    let lower = url.to_lowercase();
    BLACKLIST.iter().any(|&p| lower.contains(p))
}

/// Valida concurrentemente si los leads tienen web activa
pub async fn validate_leads(leads: &mut Vec<Lead>) -> Result<()> {
    let total = leads.len();
    println!("\n  {} Validando {} webs en paralelo...", "◈".cyan().bold(), total.to_string().yellow().bold());

    let bar = ProgressBar::new(total as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "  {spinner:.cyan} Validando [{bar:30.cyan/blue}] {pos}/{len}  ETA {eta}"
        )
        .unwrap()
        .progress_chars("█▓░")
    );

    let client = Client::builder()
        .timeout(Duration::from_secs(WEB_CHECK_TIMEOUT_SECS))
        .danger_accept_invalid_certs(true)
        .user_agent(crate::config::DEFAULT_USER_AGENT)
        .build()?;

    let mut tasks = Vec::new();

    for lead in leads.iter() {
        if let Some(url) = &lead.url_web {
            let client_c = client.clone();
            let url_c    = url.clone();
            tasks.push(tokio::spawn(async move {
                let res = match client_c.head(&url_c).send().await {
                    Ok(r)  => Ok(r),
                    Err(_) => client_c.get(&url_c).send().await,
                };
                match res {
                    Ok(r)  if r.status().is_success()  => WebStatus::Activa,
                    Ok(r)                              => WebStatus::ErrorServidor(r.status().as_u16()),
                    Err(e) if e.is_timeout()           => WebStatus::Timeout,
                    Err(e)                             => WebStatus::Caida(e.to_string()),
                }
            }));
        } else {
            tasks.push(tokio::spawn(async move { WebStatus::SinWeb }));
        }
    }

    let results = join_all(tasks).await;

    for (i, res) in results.into_iter().enumerate() {
        if let Ok(status) = res {
            let lead = &mut leads[i];
            lead.estado_web = if let Some(url) = &lead.url_web {
                if is_social_media(url) { WebStatus::SinWeb } else { status }
            } else {
                WebStatus::SinWeb
            };
        }
        bar.inc(1);
    }

    bar.finish_and_clear();

    // Resumen de validación
    let sin_web   = leads.iter().filter(|l| l.estado_web == WebStatus::SinWeb).count();
    let con_web   = leads.iter().filter(|l| l.estado_web == WebStatus::Activa).count();
    let caidos    = leads.len() - sin_web - con_web;

    println!("  {} Validación completa:", "◈".green().bold());
    println!("     {} Sin web / Red social:  {}", "•".cyan(), sin_web.to_string().green().bold());
    println!("     {} Con web activa (desc.): {}", "•".cyan(), con_web.to_string().red());
    println!("     {} Web caída/error:        {}", "•".cyan(), caidos.to_string().yellow());

    Ok(())
}
