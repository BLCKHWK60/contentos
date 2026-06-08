# ContentOS

*© 2026 ZediaTech LLC (Zedia Labs) — proprietary, source-available. Owned and operated by
Victor Chaidez. Public for inspection only; see [LICENSE](LICENSE).*

A local-first **content orchestration platform** — a Mac desktop app (Tauri 2) where every
piece of content is a *Work* that moves down a pipeline:

```
Spark → Research → Draft → Optimize (SEO/AIO) → Polish → Distribute → Ship
```

Each stage is its own station with its own AI tools. Brands are fully siloed. Everything
you create lives as plain Markdown + JSON inside a vault folder you choose — delete the app
and your whole creative history stays on disk, readable by any editor.

---

## Run it

```bash
cd contentos
chmod +x setup.sh
./setup.sh          # installs Rust + the Tauri CLI, generates icons
npm run dev         # launches the app
```

To build an installable `.app`:

```bash
npm run build       # → src-tauri/target/release/bundle/
```

First launch asks you to pick (or create) a **vault folder**. Suggested: `~/ContentOS`.
The app scaffolds:

```
ContentOS/
  brands/
    sample/ my-blog/ …    ← one folder per workspace
      workspace.json  ← workspace name, tag, accent
      works/        ← one .md per Work (frontmatter + body)
      context/      ← voice profiles, story, reference
  inbox/            ← raw captures also land here
  master/           ← reserved for the cross-brand layer
```

## The menu-bar capture

ContentOS installs a **menu-bar icon**. Click it (or `⌘⇧C`) to pop a small capture window
from anywhere — type a thought, pick a brand, hit `⌘↵`. It lands as a new Spark without
opening the full app.

---

## Turning on the AI — Settings panel

Click the **⚙ gear** (bottom-left of the masthead) to open Settings. Configure:

- **Provider** — OpenRouter (default), Anthropic direct, or any OpenAI-compatible endpoint
- **API key** — written to `vault/settings.json`, never bundled in the app
- **Endpoint** — auto-filled per provider, editable for custom routes
- **Default model** — e.g. `anthropic/claude-sonnet-4`
- **Per-stage model overrides** — run Draft on a heavy model, Polish on a cheaper one, etc.

Save and The Desk is live. Until a key is set, hitting any operation pops Settings open.
Your key lives in the vault as plain JSON — move the vault, keep your config; delete the
app, the key goes with the folder, not into any binary.

---

## Architecture notes

- **`src/index.html`** — the whole orchestration UI (brutalist editorial). The pipeline,
  its stages, and every AI operation are defined as one `PIPELINE` array near the top of
  the script. Add/remove a stage or an operation there and the rail, the filters, and The
  Desk all reshape themselves. This is how you keep the UI clean for your use case over time.
- **`src/bridge.js`** — the storage layer. Reads/writes Works as Markdown files, scaffolds
  the vault, handles the inbox. Falls back to in-memory if opened in a plain browser.
- **`src/capture.html`** — the menu-bar quick-capture window.
- **`src-tauri/src/lib.rs`** — the tray (menu-bar) icon, the capture/main window wiring.
- **`src-tauri/tauri.conf.json`** — windows, `withGlobalTauri` (so the no-build frontend can
  call `window.__TAURI__.fs` directly), bundle config.
- **`src-tauri/capabilities/default.json`** — filesystem + window permissions.

No frontend build step, no framework — single-file HTML by design, so it stays yours and
portable.

---

## Roadmap (where this goes next)

- Research stage: real web search wired into the operations.
- The Master layer: unlock cross-brand context for the "tell the whole story" moment.
- Voice memo transcription on capture.

---

## License

ContentOS is **proprietary** software, made **source-available for inspection and evaluation
only**. It is not open-source.

You may view and study the source. You may **not** copy, modify, redistribute, sell, or use it
(in whole or in part) in another product without the prior written permission of ZediaTech LLC.
"ContentOS", "Zedia Labs", and "ZediaTech" are marks of ZediaTech LLC; no trademark rights are
granted. See [LICENSE](LICENSE) for the full terms.

© 2026 ZediaTech LLC (Zedia Labs). All rights reserved. Owned and operated by Victor Chaidez.
Licensing inquiries: licensing@zedialabs.com · https://zedialabs.com
