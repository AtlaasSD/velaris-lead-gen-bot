# Velaris Lead-Gen Bot

> Extractor automático de leads de Google Maps para Velaris.  
> Identifica negocios **sin sitio web propio** que son candidatos ideales para ofrecerles desarrollo web.

---

## ¿Qué hace?

1. Busca negocios en Google Maps según zona, categoría, reseñas mínimas y calificación mínima
2. Abre el panel de cada negocio y extrae **nombre, teléfono y estado web**
3. Descarta negocios que ya tienen web corporativa propia
4. Incluye los que solo tienen redes sociales (Facebook, Instagram, WhatsApp, Linktree, etc.)
5. Genera un **reporte PDF** profesional con todos los leads calificados
6. Guarda la sesión en disco para acumular resultados entre ejecuciones

---

## Requisitos

| Herramienta | Versión mínima | Notas |
|---|---|---|
| Windows | 10 / 11 | Compatible con x64 |
| Microsoft Edge | Cualquier versión moderna | Incluido en Windows |
| Rust / Cargo | 1.75+ | El launcher lo instala si no está |

---

## 🚀 Guía de Instalación Rápida (Para Asesores)

**1. Descarga el bot:**
- Haz clic en el botón verde **"<> Code"** y luego en **"Download ZIP"**.

**2. ⚠️ MUY IMPORTANTE: ¿Dónde descomprimirlo?**
- Extrae la carpeta de archivos en tu disco local principal, por ejemplo: `C:\Velaris Lead-Gen Bot` o `C:\Velaris`.
- **NUNCA** lo guardes en *OneDrive*, *Escritorio* o *Documentos* porque bloquean los archivos del navegador de Microsoft y el bot se trabará arrojando un error extraño (`ExitStatus`).

**3. Ejecución principal:**
- Entra a la carpeta descomprimida y haz **doble clic en `run.bat`** (ícono blanco con engranaje).
- Si Windows te arroja una pantalla azul de seguridad ("Windows protegió su PC"), haz clic en la línea pequeña **"Más información"** y luego oprime el botón **"Ejecutar de todas formas"**.
- La primera vez que abras el programa, la pantalla negra se quedará instalando complementos (Rust). Esto puede tardar unos minutos. 

### 🔧 Solución a Errores Comunes
* **La pantalla negra se abre y se cierra al instante:** Ocurre porque hiciste clic en el archivo azul `run.ps1`. Debes usar siempre el que se llama **`run`** o `run.bat` (ícono blanco con engranaje).
* **Falla con "ExitStatus(21)":** El bot está colisionando con la sincronización de la nube en tu equipo. Mueve toda la carpeta del bot a un lugar sin sincronización como `C:\Velaris`.

---

## Inicio rápido

### Opción A — Ejecución Automática (Doble clic)

La forma más sencilla y garantizada de correr el bot es hacer doble clic en el archivo `run` (**run.bat**).

Este archivo se encarga de todo:
- Desbloquea las políticas de ejecución de Windows temporalmente
- Instala Rust si el equipo del asesor no lo tiene
- Compila el bot
- ¡Y lo ejecuta!
- Verifica si Rust está instalado (lo instala si no)
- Compila el proyecto en modo release (solo cuando hay cambios)
- Lanza el bot con interfaz interactiva

> **Primera ejecución:** la compilación puede tardar 2-5 minutos.  
> **Ejecuciones posteriores:** arranca en segundos.

### Opción B — Manual (para desarrolladores)

```powershell
cargo build --release
.\target\release\velaris_lead_gen.exe
```

---

## Uso del bot

Al iniciarse, el bot pide interactivamente:

| Campo | Ejemplo |
|---|---|
| Zona | `Pereira, Risaralda` |
| Tipo de negocio | Menú con 9 categorías + opción libre |
| Reseñas mínimas | `10` (0 = sin filtro) |
| Calificación mínima | `4.0` (0.0 = sin filtro) |

---

## Estructura del proyecto

```
velaris-lead-gen-bot/
├── src/
│   ├── main.rs              # Punto de entrada — interfaz TUI interactiva
│   ├── config.rs            # Constantes globales (user-agent, timeouts)
│   ├── models.rs            # Estructuras de datos (Lead, WebStatus, SearchSession)
│   ├── error.rs             # Tipo de error centralizado
│   ├── persistence.rs       # Guardado/carga de sesión JSON
│   ├── scraper/
│   │   ├── browser.rs       # Inicialización de Microsoft Edge headless
│   │   └── actions.rs       # Lógica de scraping en Google Maps
│   ├── validator/
│   │   └── web_tester.rs    # Validación concurrente de webs
│   └── reporter/
│       └── pdf_gen.rs       # Generación de reporte PDF con HTML
├── run.ps1                  # Launcher PowerShell (instala + compila + ejecuta)
├── session_leads.json       # Sesión persistente (auto-generado)
├── Cargo.toml
└── README.md
```

---

## Blacklist de dominios

Los siguientes dominios **no cuentan como sitio web corporativo** y los leads con estos links son incluidos:

`facebook.com` · `instagram.com` · `wa.me` · `whatsapp.com` · `linktr.ee` · `linktree.com` · `drive.google.com` · `docs.google.com` · `tiktok.com` · `twitter.com` · `x.com` · `youtube.com` · `t.me` · `telegram.me`

---

## Sesión persistente

El bot guarda automáticamente los leads en `session_leads.json`.  
Si ejecutas el bot varias veces con la misma zona y categoría, los resultados se **acumulan sin duplicados**.

Para **limpiar** la sesión y empezar desde cero:

```powershell
Remove-Item session_leads.json
```

---

## Tecnologías

- **Rust** — rendimiento nativo, sin GC, sin runtime externo
- **chromiumoxide** — automatización de browser via CDP (Chrome DevTools Protocol)
- **Microsoft Edge headless** — sin instalación adicional en Windows
- **reqwest** — validación concurrente de webs
- **indicatif** — barras de progreso en terminal
- **colored** — salida coloreada ANSI
- **serde / serde_json** — serialización de sesión

---

## Desarrollado por

**Velaris Agency** — Agencia de desarrollo web y marketing digital

---

*Uso interno. No distribuir sin autorización.*
