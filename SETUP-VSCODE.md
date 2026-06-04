# Setting up ContentOS in VS Code with Claude Code

A step-by-step to go from this folder to editing ContentOS with Claude Code inside VS Code.
Verified against the Claude Code docs (June 2026).

---

## 1. Prerequisites

- **VS Code 1.98.0 or newer** (check Help → About). Update if older.
- **Node.js** (LTS) — https://nodejs.org
- **Rust** — installed automatically by `./setup.sh`, or via https://rustup.rs
- **Xcode Command Line Tools** (macOS) — `./setup.sh` triggers the install if missing.
- A **Claude Pro or Max** subscription (or Anthropic Console account) to sign into Claude Code.

---

## 2. Get the project onto your machine

```bash
# from wherever you downloaded it
tar -xzf contentos-tauri.tar.gz
mv contentos ~/Nexus/   # or wherever you keep projects
cd ~/Nexus/contentos
```

(Optional but recommended) make it a git repo so Claude Code's checkpoints and commits work:

```bash
git init
git add .
git commit -m "ContentOS v0.3 — initial"
```

The included `.gitignore` already keeps `node_modules`, build output, and generated icons out.

---

## 3. Install the Claude Code extension

1. Open VS Code.
2. Open the Extensions view: `Cmd+Shift+X` (Mac) / `Ctrl+Shift+X` (Win/Linux).
3. Search **"Claude Code"**. Install the official one published by **Anthropic**.
4. If the icon doesn't show up, run **Developer: Reload Window** from the Command Palette
   (`Cmd+Shift+P`).

The extension bundles the Claude Code CLI — you don't need a separate install. You can still
run `claude` in VS Code's integrated terminal (`` Cmd+` ``) for CLI-only features.

---

## 4. Open the project and sign in

1. **File → Open Folder…** and choose the `contentos` folder. Open the *folder*, not a single
   file — Claude Code needs the project root (where `CLAUDE.md` lives) for full context.
2. Click the **Spark icon** to open Claude Code:
   - Editor Toolbar (top-right of an open file), or
   - the Spark icon in the left Activity Bar, or
   - **✱ Claude Code** in the bottom-right Status Bar.
3. On first open, click **Sign in** and authorize in your browser.

Claude Code automatically reads `CLAUDE.md` at the project root — that file tells it the
architecture, the constraints, and the design system, so it won't, for example, reach for a
framework or break the single-file pattern.

---

## 5. First moves (recommended habits)

- **Start in Plan mode** for anything non-trivial. Click the mode indicator at the bottom of
  the prompt box → Plan. Claude writes out what it intends to do as a Markdown doc you approve
  before any file is touched. Switch to Normal (asks before each edit) or Auto-accept once you
  trust a task.
- **@-mention files** instead of pasting. Type `@index.html` (fuzzy match works) to give Claude
  the file. `@src/` with a trailing slash for a folder.
- **Run `/terminal-setup` once** (in a Claude session) so `Shift+Enter` makes a newline in the
  integrated terminal instead of submitting — makes multi-line prompts painless.
- **Review every diff.** Claude shows side-by-side changes; accept, reject, or edit in place.
- **Use checkpoints.** Hover a message → rewind to revert file changes if a direction goes wrong.

---

## 6. Good opening prompts for this project

```
Read CLAUDE.md, then give me a 5-line summary of the architecture so I know you've got it.

In @index.html, add a new pipeline stage "Repurpose" between Distribute and Ship with
three operations: turn into a thread, turn into a newsletter blurb, turn into a carousel
outline. Follow the PIPELINE-is-data rule in CLAUDE.md. Show me a plan first.

Wire a "Test connection" button into the Settings panel that pings the configured provider
and reports success/failure. Keep the brutalist styling.
```

Because `CLAUDE.md` spells out the constraints, you can keep prompts short — Claude already
knows not to add a build step, not to use localStorage, and to keep brands siloed.

---

## 7. Wiring your AI key (one-time, in the app — not the code)

After `npm run dev`, in ContentOS click the **⚙ gear** → set Provider to **OpenRouter**, paste
your key, set a default model (e.g. `anthropic/claude-sonnet-4`), Save. The key is written to
your vault's `settings.json`, never into the repo. (This is separate from your Claude Code
sign-in, which is your Anthropic subscription.)

---

## 8. Troubleshooting

- **Spark icon missing:** open a file (icon needs one), check VS Code ≥ 1.98.0, reload window,
  disable other AI extensions if they conflict.
- **Stuck on sign-in with `ANTHROPIC_API_KEY` set:** launch VS Code from a terminal with
  `code .` so it inherits your shell env, or just sign in with your Claude account.
- **`Cmd+Esc` does nothing (macOS Tahoe+):** the system Game Overlay grabs it — clear it in
  System Settings → Keyboard → Keyboard Shortcuts → Game Controllers.
- **Tauri build fails:** confirm Xcode CLT (`xcode-select -p`) and Rust (`cargo --version`).

---

## Reference

- Claude Code in VS Code: https://code.claude.com/docs/en/vs-code
- Claude Code overview: https://docs.claude.com/en/docs/claude-code/overview
- Common workflows: https://code.claude.com/docs/en/common-workflows
