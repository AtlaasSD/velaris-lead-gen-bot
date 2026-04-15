use chromiumoxide::Page;
use crate::models::{Lead, WebStatus};
use crate::error::Result;

/// Genera un PDF profesional con los leads extraídos
pub async fn generate_report(page: &Page, leads: &[Lead], output_path: &str) -> Result<()> {
    let date_str = chrono::Local::now().format("%d/%m/%Y %H:%M").to_string();
    let total = leads.len();

    // ── Tabla de filas ────────────────────────────────────────────────────────
    let rows: String = leads.iter().map(|lead| {
        let telefono = lead.telefono.as_deref().unwrap_or("—");
        let direccion = lead.direccion.as_deref().unwrap_or("—");
        let web_badge = match &lead.url_web {
            Some(url) => format!(
                r#"<span class="badge social">Red Social<br><small>{}</small></span>"#,
                url.chars().take(35).collect::<String>()
            ),
            None => r#"<span class="badge none">Sin web</span>"#.to_string(),
        };
        let estado_badge = match &lead.estado_web {
            WebStatus::SinWeb         => r#"<span class="badge none">Sin web</span>"#,
            WebStatus::Activa         => r#"<span class="badge ok">Activa</span>"#,
            WebStatus::Timeout        => r#"<span class="badge warn">Timeout</span>"#,
            WebStatus::ErrorServidor(_) => r#"<span class="badge warn">Error servidor</span>"#,
            WebStatus::Caida(_)       => r#"<span class="badge warn">Caída</span>"#,
        };
        format!(
            r#"<tr>
                <td class="name">{}</td>
                <td><code>{}</code></td>
                <td>{}</td>
                <td class="small">{}</td>
                <td>{} ★</td>
                <td>{}</td>
                <td>{}</td>
            </tr>"#,
            lead.nombre, telefono, web_badge, direccion,
            lead.calificacion, lead.reseñas, estado_badge
        )
    }).collect::<Vec<_>>().join("\n");

    let html = format!(r#"<!DOCTYPE html>
<html lang="es">
<head>
<meta charset="UTF-8"/>
<style>
  @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap');
  * {{ box-sizing: border-box; margin: 0; padding: 0; }}
  body {{ font-family: 'Inter', sans-serif; color: #1a1a2e; background: #fff; padding: 32px; font-size: 12px; }}

  .header {{ display: flex; align-items: center; justify-content: space-between; margin-bottom: 28px; border-bottom: 3px solid #2563eb; padding-bottom: 16px; }}
  .header h1 {{ font-size: 22px; font-weight: 700; color: #2563eb; letter-spacing: -0.5px; }}
  .header .meta {{ text-align: right; color: #6b7280; font-size: 11px; line-height: 1.6; }}

  .stats {{ display: flex; gap: 16px; margin-bottom: 24px; }}
  .stat {{ flex: 1; background: #f0f4ff; border-radius: 10px; padding: 14px 18px; border-left: 4px solid #2563eb; }}
  .stat .val {{ font-size: 26px; font-weight: 700; color: #2563eb; }}
  .stat .label {{ font-size: 10px; color: #6b7280; text-transform: uppercase; letter-spacing: 0.5px; margin-top: 2px; }}

  table {{ width: 100%; border-collapse: collapse; margin-top: 8px; }}
  thead tr {{ background: #2563eb; color: white; }}
  thead th {{ padding: 10px 12px; text-align: left; font-weight: 600; font-size: 11px; letter-spacing: 0.3px; }}
  tbody tr:nth-child(even) {{ background: #f8faff; }}
  tbody tr:hover {{ background: #e8f0fe; }}
  td {{ padding: 9px 12px; border-bottom: 1px solid #e5e7eb; vertical-align: top; }}
  td.name {{ font-weight: 600; color: #111827; }}
  td.small {{ font-size: 10px; color: #6b7280; max-width: 160px; }}
  code {{ background: #f3f4f6; padding: 2px 6px; border-radius: 4px; font-size: 11px; color: #1d4ed8; }}

  .badge {{ display: inline-block; padding: 3px 8px; border-radius: 12px; font-size: 10px; font-weight: 600; text-align: center; }}
  .badge.none {{ background: #dbeafe; color: #1d4ed8; }}
  .badge.social {{ background: #fef3c7; color: #92400e; }}
  .badge.ok {{ background: #d1fae5; color: #065f46; }}
  .badge.warn {{ background: #fee2e2; color: #991b1b; }}

  .footer {{ margin-top: 24px; text-align: center; color: #9ca3af; font-size: 10px; border-top: 1px solid #e5e7eb; padding-top: 12px; }}
</style>
</head>
<body>

<div class="header">
  <div>
    <h1>⚡ Velaris Lead-Gen Report</h1>
    <div style="color:#6b7280;margin-top:4px;">Negocios sin sitio web propio — oportunidades de venta</div>
  </div>
  <div class="meta">
    <strong>{categoria}</strong> en <strong>{zona}</strong><br>
    Generado el {date}<br>
    {total} leads calificados
  </div>
</div>

<div class="stats">
  <div class="stat"><div class="val">{total}</div><div class="label">Leads Totales</div></div>
  <div class="stat"><div class="val">{con_tel}</div><div class="label">Con Teléfono</div></div>
  <div class="stat"><div class="val">{sin_web}</div><div class="label">Sin Sitio Web</div></div>
  <div class="stat"><div class="val">{solo_redes}</div><div class="label">Solo Redes Sociales</div></div>
</div>

<table>
  <thead>
    <tr>
      <th>Negocio</th>
      <th>Teléfono</th>
      <th>Web / Redes</th>
      <th>Dirección</th>
      <th>Cal.</th>
      <th>Reseñas</th>
      <th>Estado</th>
    </tr>
  </thead>
  <tbody>
    {rows}
  </tbody>
</table>

<div class="footer">Velaris Agency — Reporte generado automáticamente. Uso interno.</div>
</body>
</html>"#,
        categoria = leads.first().map(|l| l.categoria.as_str()).unwrap_or("—"),
        zona = leads.first().map(|l| l.categoria.as_str()).unwrap_or("—"),
        date = date_str,
        total = total,
        con_tel = leads.iter().filter(|l| l.telefono.is_some()).count(),
        sin_web = leads.iter().filter(|l| l.url_web.is_none()).count(),
        solo_redes = leads.iter().filter(|l| l.url_web.is_some()).count(),
        rows = rows,
    );

    page.set_content(&html).await
        .map_err(|e| crate::error::BotError::BrowserError(e.to_string()))?;

    let pdf_data = page.pdf(Default::default()).await
        .map_err(|e| crate::error::BotError::PdfError(e.to_string()))?;

    std::fs::write(output_path, pdf_data)
        .map_err(|e| crate::error::BotError::PersistenceError(e.to_string()))?;

    Ok(())
}
