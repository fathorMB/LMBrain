# LMBrain — Piano implementazione: supporto Codex CLI (parità con Claude Code)

> Stato: **proposto / decisioni bloccate**, non ancora implementato.
> Obiettivo: rendere LMBrain agent-agnostico estendendo a **OpenAI Codex CLI** ciò che già
> produce per Claude Code — registrazione dell'MCP, lancio/monitoraggio sessioni, e istruzioni
> dell'agente. Documento guida per il dev agent.

## Cosa LMBrain produce oggi per Claude Code (baseline)

1. **Registrazione MCP**: all'apertura del workspace l'app scrive `.mcp.json` alla root
   (`src-tauri/src/commands/mcp_registration.rs`) con
   `{ "mcpServers": { "lmbrain": { "command": <bin>, "args": ["--root", <ws>] } } }`.
   Merge idempotente che preserva altri server. Il comando si risolve via
   `LMBRAIN_MCP_BIN` → binario accanto all'eseguibile → `PATH` (`resolve_mcp_command()`).
2. **Binario MCP**: `lmbrain-mcp` (JSON-RPC su stdio, `protocolVersion` "2024-11-05";
   `initialize` / `tools/list` / `tools/call`).
3. **Istruzioni agente**: `.lmbrain/AGENT.md` + il bootstrap prompt in
   `.lmbrain/templates/project-lead-bootstrap-prompt.md`.
4. **Launcher Sessions**: view che lancia `claude` / `ollama launch claude --model X` in PTY
   (`src-tauri/src/commands/sessions.rs`, `src/components/Sessions/`).

## Decisioni prese (confermate con l'operatore)

- **MCP per Codex** → **project-scoped `.codex/config.toml`** alla root del workspace, con
  `[mcp_servers.lmbrain]`, **e** garanzia del trust del workspace nel config utente. Non si
  tocca/duplica il config globale (che è ricco e personale).
- **Sessions per Codex** → **solo Codex nativo** (no `ollama launch codex` in v1).
- **Scope v1** → tutti e quattro: (A) registrazione MCP, (B) launcher Codex, (C) scaffolding
  `AGENTS.md`, (D) aggiornamento kit & docs.

## Riscontri reali sulla macchina dell'operatore (da verificare lato dev)

- **Codex è installato e molto usato, ma NON è nel `PATH`.** Il binario reale è
  `C:\Users\moren\AppData\Local\OpenAI\Codex\bin\<hash>\codex.exe` (CLI del Codex desktop;
  in `~/.codex/config.toml` compare come `CODEX_CLI_PATH`). → la risoluzione del binario è
  obbligatoria, non si può assumere `codex` in `PATH`.
- **`e:\git\lmbrain` è già `trusted`** in `~/.codex/config.toml` (`[projects.'e:\git\lmbrain']`
  `trust_level = "trusted"`). Le chiavi `[projects.<path>]` esistenti hanno casing misto
  (`e:\git\…` e `E:\Git\…`) → attenzione alla normalizzazione del path nella chiave di trust.
- Il config globale contiene già `[mcp_servers.*]` (`node_repl`, `loomle`, `unreal`),
  `model = "gpt-5.4"`, plugin e voci `[projects.*]`. **Da preservare integralmente** se mai si
  scrive nel config utente.
- `CODEX_HOME` = `~/.codex` (rispettare la env var `CODEX_HOME` se impostata).

## Fatti su Codex (dalla doc ufficiale, giugno 2026)

- MCP via `config.toml` in tabelle `[mcp_servers.<name>]`. Stdio: `command` (richiesto),
  `args`, `env` (sotto-tabella `[mcp_servers.<name>.env]`), `cwd`, `startup_timeout_sec`
  (default 10), `tool_timeout_sec` (default 60), `enabled`, `required`.
- **Config project-scoped `.codex/config.toml` supportato, ma SOLO per progetti trusted.** Il
  trust è nel config utente: `[projects.<path>] trust_level = "trusted"`. I config di progetto
  **non** possono sovrascrivere provider/auth/selezione-profilo/telemetria — ma `mcp_servers`
  **sì**.
- Esiste `codex mcp add <name> --env K=V -- <command> <args…>` (scrive nel config, tipicamente
  globale) — **non** la via scelta qui.
- Istruzioni: `AGENTS.md` (root repo) con catena di precedenza
  (`~/.codex/AGENTS.md` → `<git-root>/AGENTS.md` → `<cwd>/AGENTS.md`, più varianti
  `AGENTS.override.md`). `project_doc_fallback_filenames` = nomi-file alternativi (sono
  *filename*, non path). `project_doc_max_bytes` (default 32 KiB).

Esempio TOML stdio (verbatim dalla doc):
```toml
[mcp_servers.context7]
command = "npx"
args = ["-y", "@upstash/context7-mcp"]
env_vars = ["LOCAL_TOKEN"]

[mcp_servers.context7.env]
MY_ENV_VAR = "MY_ENV_VALUE"
```

## Architettura proposta

### A. Registrazione MCP per Codex — `commands/codex_registration.rs`
Modellato su `mcp_registration.rs`. All'apertura del workspace (in `open_workspace`, accanto
alla scrittura di `.mcp.json`):

1. **Scrive/aggiorna `<workspace>/.codex/config.toml`** con:
   ```toml
   [mcp_servers.lmbrain]
   command = "<resolved lmbrain-mcp>"
   args = ["--root", "<workspace-absolute>"]
   ```
   - Usare **`toml_edit`** (preserva formattazione/commenti/altre tabelle) per il merge
     idempotente; riscrivere solo se il contenuto cambia (come fa già `register_mcp_server`).
   - Riusare `resolve_mcp_command()` per il `command`.
2. **Garantisce il trust** del workspace nel config utente (`$CODEX_HOME/config.toml`,
   default `~/.codex/config.toml`):
   - Se manca, aggiungere `[projects.'<workspace>'] trust_level = "trusted"`.
   - **Non** modificare voci esistenti; preservare l'intero file con `toml_edit`.
   - Replicare **esattamente** la convenzione di path-key di Codex (vedi rischio sotto).
   - Best-effort: non bloccare l'apertura del workspace se la scrittura fallisce.

### B. Scaffolding `AGENTS.md`
Generare/aggiornare un **`AGENTS.md` alla root del repo** (fuori da `.lmbrain/` — è la
convenzione nativa di Codex) con un **blocco gestito** delimitato
(`<!-- lmbrain:begin -->` … `<!-- lmbrain:end -->`) per fare merge senza sovrascrivere
contenuto utente preesistente. Il blocco indirizza Codex a operare come il Project Lead
descritto in `.lmbrain/AGENT.md` e a rispettare `CONTRACT.md` / `QUALITY.md`.
- Idempotente; riscrive solo il blocco gestito.
- Tenere il contenuto sotto `project_doc_max_bytes` (32 KiB): nel blocco mettere un **puntatore**
  conciso, non l'intero AGENT.md.

### C. Launcher Codex nativo nelle Sessions
- **Backend** (`sessions.rs`): estendere `SessionMode` con `Codex`. In `build_command`, per
  `Codex` usare un nuovo `resolve_codex_command()` (vedi sotto), `cwd` = root workspace, env
  ereditato; nessun `--model` (Codex nativo usa il `model` del suo config.toml).
- **`resolve_codex_command()`** (ordine): `LMBRAIN_CODEX_BIN` → path configurato in Settings →
  auto-detect del più recente `%LOCALAPPDATA%\OpenAI\Codex\bin\*\codex.exe` →
  `codex`/`codex.exe` in `PATH`.
- **Frontend** (`SessionsView.tsx`): terzo `ModeButton` "Codex" accanto a Claude/Ollama; in
  modalità Codex niente dropdown modello. Etichetta di default "Codex". Aggiornare i tipi
  (`SessionMode = "claude" | "ollama" | "codex"`) in `src/types/index.ts` e `models/session.rs`.

### D. Kit & docs
- `kit/.lmbrain/mcp/`: aggiungere un esempio `codex-config.toml` accanto a `lmbrain-mcp.json`;
  aggiornare `mcp/README.md` per spiegare la registrazione Codex (project `.codex/config.toml`
  + trust).
- `OPERATOR.md` / `README.md`: documentare il supporto multi-agente (Claude e Codex).
- View **Agents & MCP** (`src/components/Agents/AgentsMCPView.tsx`): riflettere che l'MCP è
  registrato sia per Claude (`.mcp.json`) sia per Codex (`.codex/config.toml`).

## Nuove dipendenze
- Rust: **`toml_edit`** (merge format-preserving di `.codex/config.toml` e del config utente).

## Fasi (incrementali, ognuna verde su `cargo test`/`clippy`/`pnpm test`/`pnpm lint`)
1. **`codex_registration.rs`** headless: scrittura/merge di `.codex/config.toml` di progetto +
   ensure-trust nel config utente, con test (merge idempotente, preservazione di tabelle/voci
   esistenti, path-key del trust). Cablare in `open_workspace`.
2. **Validazione MCP↔Codex**: avviare `codex` reale nel workspace e verificare che i tool
   `lmbrain-*` siano effettivamente esposti (vedi rischi). Correggere `lmbrain-mcp` se Codex usa
   un handshake/notifiche che il server attuale non gestisce.
3. **Launcher Codex** nelle Sessions (`resolve_codex_command` + `SessionMode::Codex` + UI).
4. **`AGENTS.md`** scaffolding (blocco gestito).
5. **Kit & docs** + ADR (estende ADR-001/ADR-006: scrittura di config esterni alla repo —
   `~/.codex/config.toml` — è una nuova superficie) + CHANGELOG. Bump versione lasciato
   all'operatore.

## Rischi / da validare presto
- **`mcp_servers` project-scoped davvero caricati.** Storicamente c'è stato un caso
  (openai/codex#3441) in cui i server in `config.toml` non venivano usati. Validare con la
  versione installata che i tool `lmbrain-*` compaiano in `codex` aprendo il workspace.
- **Normalizzazione del path nella chiave di trust.** Le voci `[projects.<path>]` esistenti
  hanno casing misto; va replicata la convenzione esatta che Codex usa per il lookup (probabile
  case-insensitive su Windows, ma da confermare) per evitare doppioni o trust non riconosciuto.
- **`lmbrain-mcp` e le notifiche JSON-RPC.** Il server attuale risponde a ogni metodo; per i
  metodi `notifications/*` (es. `notifications/initialized`) JSON-RPC **non** prevede risposta.
  Verificare che Codex non si rompa; eventualmente far sì che `lmbrain-mcp` non risponda alle
  notifiche e gestisca un eventuale `protocolVersion` più recente.
- **Risoluzione di `codex.exe`** robusta su path con hash che cambia tra versioni (auto-detect
  "più recente"); fornire override via Settings/env.
- **`AGENTS.md` fuori da `.lmbrain/`.** Devia dalla filosofia "tutto in `.lmbrain/`"; usare un
  blocco gestito e non sovrascrivere contenuto utente; valutare un toggle nelle Settings.
- **Scrittura nel config utente (`~/.codex/config.toml`).** È l'unico write globale: solo
  aggiunta del trust se assente, mai modifica di altro, con backup/atomic-write e trasparenza
  verso l'operatore.

## Out of scope (v1)
- `ollama launch codex` (Codex via Ollama).
- Profili Codex (`--profile`), `model_providers` custom, `AGENTS.override.md`.
- Refactor del modello `SessionMode` in (agente × launcher): per ora `codex` è una terza
  variante; la generalizzazione si valuta dopo.
