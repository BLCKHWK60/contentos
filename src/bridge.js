/* ============================================================
   ContentOS — storage bridge
   In the Tauri app this reads/writes real files under a vault
   folder you choose. In a plain browser it no-ops so the UI
   still runs from memory. Everything lives as plain files:

     {vault}/brands/{brand}/works/{id}.md     (frontmatter + body)
     {vault}/brands/{brand}/profile.json      (voice profile)
     {vault}/inbox/                           (raw captures land here too)

   Delete the app and your whole creative history remains as
   readable Markdown + JSON in that folder.
   ============================================================ */
(function () {
  const T = window.__TAURI__;
  const isTauri = !!(T && T.fs);
  const BRANDS = ["victor", "bricks", "zedia", "zediatech"];

  const COS = {
    isTauri,
    vault: null,

    // ---- vault selection (persisted in app config) ----
    async ensureVault() {
      if (!isTauri) return null;
      const { fs, dialog, path } = T;
      const cfgDir = await path.appConfigDir();
      await fs.mkdir(cfgDir, { recursive: true }).catch(() => {});
      const ptr = await path.join(cfgDir, "vault.txt");

      if (await fs.exists(ptr).catch(() => false)) {
        this.vault = (await fs.readTextFile(ptr)).trim();
      }
      if (!this.vault || !(await fs.exists(this.vault).catch(() => false))) {
        const picked = await dialog.open({
          directory: true,
          multiple: false,
          title: "Choose (or create) your ContentOS vault folder",
        });
        if (!picked) return null;
        this.vault = Array.isArray(picked) ? picked[0] : picked;
        await fs.writeTextFile(ptr, this.vault);
      }
      await this.scaffold();
      return this.vault;
    },

    async scaffold() {
      const { fs, path } = T;
      await fs.mkdir(await path.join(this.vault, "inbox"), { recursive: true }).catch(() => {});
      await fs.mkdir(await path.join(this.vault, "master"), { recursive: true }).catch(() => {});
      await fs.mkdir(await path.join(this.vault, "published"), { recursive: true }).catch(() => {});
      for (const b of BRANDS) {
        await fs.mkdir(await path.join(this.vault, "brands", b, "works"), { recursive: true }).catch(() => {});
        await fs.mkdir(await path.join(this.vault, "brands", b, "context"), { recursive: true }).catch(() => {});
      }
    },

    // ---- works ----
    async loadWorks() {
      if (!isTauri || !this.vault) return null;
      const { fs, path } = T;
      const out = {};
      for (const b of BRANDS) {
        out[b] = [];
        const dir = await path.join(this.vault, "brands", b, "works");
        let entries = [];
        try { entries = await fs.readDir(dir); } catch (e) { entries = []; }
        for (const e of entries) {
          if (!e.name || !e.name.endsWith(".md")) continue;
          try {
            const raw = await fs.readTextFile(await path.join(dir, e.name));
            out[b].push(parseWork(raw, e.name.replace(/\.md$/, "")));
          } catch (e2) {}
        }
        out[b].sort((x, y) => (y._mtime || 0) - (x._mtime || 0));
      }
      return out;
    },

    async saveWork(brand, w) {
      if (!isTauri || !this.vault) return;
      const { fs, path } = T;
      const dir = await path.join(this.vault, "brands", brand, "works");
      await fs.mkdir(dir, { recursive: true }).catch(() => {});
      await fs.writeTextFile(await path.join(dir, w.id + ".md"), serializeWork(w));
    },

    async appendCapture(brand, w) {
      // Save as a work (Spark) AND drop a copy in the shared inbox.
      await this.saveWork(brand, w);
      if (!isTauri || !this.vault) return;
      const { fs, path } = T;
      const stamp = new Date().toISOString().replace(/[:.]/g, "-");
      const file = await path.join(this.vault, "inbox", brand + "-" + stamp + ".md");
      await fs.writeTextFile(file, (w.title ? "# " + w.title + "\n\n" : "") + stripHtml(w.body));
    },

    async saveProfile(brand, data) {
      if (!isTauri || !this.vault) return;
      const { fs, path } = T;
      const dir = await path.join(this.vault, "brands", brand);
      await fs.mkdir(dir, { recursive: true }).catch(() => {});
      await fs.writeTextFile(await path.join(dir, "profile.json"), JSON.stringify(data, null, 2));
    },

    async loadSettings() {
      if (!isTauri || !this.vault) return null;
      const { fs, path } = T;
      const p = await path.join(this.vault, "settings.json");
      if (await fs.exists(p).catch(() => false)) {
        try { return JSON.parse(await fs.readTextFile(p)); } catch (e) { return null; }
      }
      return null;
    },

    async saveSettings(data) {
      if (!isTauri || !this.vault) return;
      const { fs, path } = T;
      await fs.writeTextFile(await path.join(this.vault, "settings.json"), JSON.stringify(data, null, 2));
    },

    // ---- UI preferences (layout widths, collapsed flags, guide-seen) ----
    async loadUI() {
      if (!isTauri || !this.vault) return null;
      const { fs, path } = T;
      const p = await path.join(this.vault, "ui.json");
      if (await fs.exists(p).catch(() => false)) {
        try { return JSON.parse(await fs.readTextFile(p)); } catch (e) { return null; }
      }
      return null;
    },
    async saveUI(data) {
      if (!isTauri || !this.vault) return;
      const { fs, path } = T;
      await fs.writeTextFile(await path.join(this.vault, "ui.json"), JSON.stringify(data, null, 2));
    },

    // ---- pipeline (custom workflow: stages, tasks, instructions, house-style notes) ----
    async loadPipeline() {
      if (!isTauri || !this.vault) return null;
      const { fs, path } = T;
      const p = await path.join(this.vault, "pipeline.json");
      if (await fs.exists(p).catch(() => false)) {
        try { return JSON.parse(await fs.readTextFile(p)); } catch (e) { return null; }
      }
      return null;
    },
    async savePipeline(data) {
      if (!isTauri || !this.vault) return;
      const { fs, path } = T;
      await fs.writeTextFile(await path.join(this.vault, "pipeline.json"), JSON.stringify(data, null, 2));
    },

    // ---- voice profile ----
    async loadVoice(brand) {
      if (!isTauri || !this.vault) return null;
      const { fs, path } = T;
      const p = await path.join(this.vault, "brands", brand, "context", "voice-profile.json");
      if (await fs.exists(p).catch(() => false)) {
        try { return JSON.parse(await fs.readTextFile(p)); } catch (e) { return null; }
      }
      return null;
    },
    async saveVoice(brand, obj) {
      if (!isTauri || !this.vault) return;
      const { fs, path } = T;
      const dir = await path.join(this.vault, "brands", brand, "context");
      await fs.mkdir(dir, { recursive: true }).catch(() => {});
      await fs.writeTextFile(await path.join(dir, "voice-profile.json"), JSON.stringify(obj, null, 2));
    },

    // ---- context library (markdown files) ----
    async loadContext(brand) {
      if (!isTauri || !this.vault) return null;
      const { fs, path } = T;
      const dir = await path.join(this.vault, "brands", brand, "context");
      let entries = [];
      try { entries = await fs.readDir(dir); } catch (e) { return []; }
      const out = [];
      for (const e of entries) {
        if (!e.name || !e.name.endsWith(".md")) continue;
        try { out.push({ name: e.name.replace(/\.md$/, ""), body: await fs.readTextFile(await path.join(dir, e.name)) }); } catch (e2) {}
      }
      return out;
    },
    async saveContextFile(brand, name, body) {
      if (!isTauri || !this.vault) return;
      const { fs, path } = T;
      const dir = await path.join(this.vault, "brands", brand, "context");
      await fs.mkdir(dir, { recursive: true }).catch(() => {});
      const safe = name.replace(/[^\w\- ]/g, "").trim() || "note";
      await fs.writeTextFile(await path.join(dir, safe + ".md"), body || "");
    },
    async deleteContextFile(brand, name) {
      if (!isTauri || !this.vault) return;
      const { fs, path } = T;
      const safe = name.replace(/[^\w\- ]/g, "").trim();
      const p = await path.join(this.vault, "brands", brand, "context", safe + ".md");
      try { await fs.remove(p); } catch (e) {}
    },

    // ---- master context (cross-brand, lives in /master) ----
    async loadMaster() {
      if (!isTauri || !this.vault) return null;
      const { fs, path } = T;
      const dir = await path.join(this.vault, "master");
      let entries = [];
      try { entries = await fs.readDir(dir); } catch (e) { return []; }
      const out = [];
      for (const e of entries) {
        if (!e.name || !e.name.endsWith(".md")) continue;
        try { out.push({ name: e.name.replace(/\.md$/, ""), body: await fs.readTextFile(await path.join(dir, e.name)) }); } catch (e2) {}
      }
      return out;
    },
    async saveMasterFile(name, body) {
      if (!isTauri || !this.vault) return;
      const { fs, path } = T;
      const dir = await path.join(this.vault, "master");
      await fs.mkdir(dir, { recursive: true }).catch(() => {});
      const safe = name.replace(/[^\w\- ]/g, "").trim() || "note";
      await fs.writeTextFile(await path.join(dir, safe + ".md"), body || "");
    },
    async deleteMasterFile(name) {
      if (!isTauri || !this.vault) return;
      const { fs, path } = T;
      const safe = name.replace(/[^\w\- ]/g, "").trim();
      const p = await path.join(this.vault, "master", safe + ".md");
      try { await fs.remove(p); } catch (e) {}
    },

    // ---- published (clean Markdown export of shipped works) ----
    async savePublished(brand, w) {
      if (!isTauri || !this.vault) return;
      const { fs, path } = T;
      const dir = await path.join(this.vault, "published", brand);
      await fs.mkdir(dir, { recursive: true }).catch(() => {});
      const body = stripHtml(w.body);
      const md = (w.title ? "# " + w.title + "\n\n" : "") + body;
      const name = slugify(w.title) || slugify(body.split("\n")[0]) || w.id;
      await fs.writeTextFile(await path.join(dir, name + ".md"), md);
      return name + ".md";
    },

    // ---- export a finished work to a file under published/{brand} ----
    async saveExport(brand, name, ext, content) {
      if (!isTauri || !this.vault) return null;
      const { fs, path } = T;
      const dir = await path.join(this.vault, "published", brand);
      await fs.mkdir(dir, { recursive: true }).catch(() => {});
      const safe = slugify(name) || "export";
      const file = await path.join(dir, safe + "." + (ext || "md"));
      await fs.writeTextFile(file, content || "");
      return file;
    },

    // ---- publish log (what was published where, when, returned URL) ----
    async loadPublishLog() {
      if (!isTauri || !this.vault) return [];
      const { fs, path } = T;
      const p = await path.join(this.vault, "published-log.json");
      if (await fs.exists(p).catch(() => false)) {
        try { return JSON.parse(await fs.readTextFile(p)); } catch (e) { return []; }
      }
      return [];
    },
    async savePublishLog(list) {
      if (!isTauri || !this.vault) return;
      const { fs, path } = T;
      await fs.writeTextFile(await path.join(this.vault, "published-log.json"), JSON.stringify(list || [], null, 2));
    },

    // ---- reveal the vault folder in the OS file browser ----
    async revealVault() {
      if (!isTauri || !this.vault) return false;
      // Preferred: opener plugin (handles raw filesystem paths). Fall back to shell.
      try {
        if (T.opener && T.opener.openPath) { await T.opener.openPath(this.vault); return true; }
      } catch (e) {}
      try {
        if (T.shell && T.shell.open) { await T.shell.open(this.vault); return true; }
      } catch (e) {}
      return false;
    },

    // ---- switch to a different vault folder (user-initiated) ----
    async changeVault() {
      if (!isTauri) return null;
      const { fs, dialog, path } = T;
      const picked = await dialog.open({
        directory: true,
        multiple: false,
        title: "Choose a different ContentOS vault folder",
      });
      if (!picked) return null;
      this.vault = Array.isArray(picked) ? picked[0] : picked;
      const cfgDir = await path.appConfigDir();
      await fs.mkdir(cfgDir, { recursive: true }).catch(() => {});
      await fs.writeTextFile(await path.join(cfgDir, "vault.txt"), this.vault);
      await this.scaffold();
      return this.vault;
    },
  };

  // ---- (de)serialization: frontmatter + body ----
  function serializeWork(w) {
    const fm = [
      "---",
      "id: " + w.id,
      "title: " + (w.title || "").replace(/\n/g, " "),
      "stage: " + w.stage,
      "platform: " + (w.platform || "medium"),
      "updated: " + (w.updated || new Date().toISOString()),
      "---",
      "",
    ].join("\n");
    return fm + (w.body || "");
  }

  function parseWork(raw, fallbackId) {
    const m = raw.match(/^---\n([\s\S]*?)\n---\n?/);
    const meta = {};
    let body = raw;
    if (m) {
      m[1].split("\n").forEach((line) => {
        const i = line.indexOf(":");
        if (i > -1) meta[line.slice(0, i).trim()] = line.slice(i + 1).trim();
      });
      body = raw.slice(m[0].length);
    }
    return {
      id: meta.id || fallbackId,
      title: meta.title || "",
      stage: meta.stage || "spark",
      platform: meta.platform || "medium",
      updated: meta.updated || "—",
      body: body,
      _mtime: Date.parse(meta.updated) || 0,
    };
  }

  function stripHtml(s) {
    const d = document.createElement("div");
    d.innerHTML = s || "";
    return d.innerText;
  }

  function slugify(s) {
    return (s || "")
      .toString()
      .trim()
      .toLowerCase()
      .replace(/[#*_`>\[\]()]/g, "")
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "")
      .slice(0, 80);
  }

  window.COS = COS;
})();
