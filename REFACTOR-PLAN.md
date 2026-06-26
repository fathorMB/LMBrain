# LMBrain — Piano di consolidamento del codice (app + strumenti)

> Scopo: rendere il codice della soluzione **più solido, pulito ed efficiente**, senza
> intervenire sul kit LLM vero e proprio. Coinvolge `lmbrain-core`, `lmbrain-mcp`,
> `src-tauri` (backend desktop) e `src/` (frontend React).
>
> Stato: **proposto**. Esecuzione prevista in sequenza (Fase 1 → 5), con revisione a
> fine di ogni fase. Ogni fase è indipendente e deve restare verde su
> `cargo test` (workspace) + `pnpm test` prima di passare alla successiva.

---

## Valutazione di partenza

L'architettura è **sana** e non richiede riscritture. I confini sono corretti:

- `lmbrain-core` — logica di mutazione pura, filesystem-backed, senza dipendenze Tauri.
- `lmbrain-mcp` — server JSON-RPC stdio che espone i verbi di mutazione agli agenti.
- `src-tauri` — backend desktop: lettura artefatti, watcher, git, path-safety, comandi Tauri.
- `src/` — frontend React/TypeScript.

I problemi sono di **consistenza interna, duplicazione e leggibilità**, non di design.
Tre temi dominanti (in ordine di impatto) più una serie di rilievi minori ma concreti.

---

## Tema 1 — Due parser di frontmatter divergenti (rischio di correttezza)

Esistono due implementazioni indipendenti che leggono gli stessi file `.md`:

- [`src-tauri/src/commands/parser.rs`](src-tauri/src/commands/parser.rs) — usa `serde_yaml`,
  parsing YAML completo. È ciò con cui il desktop **legge** gli artefatti.
- [`lmbrain-core/src/frontmatter.rs`](lmbrain-core/src/frontmatter.rs) — parser custom
  line-based (`Document`). È ciò con cui il core **scrive** e rilegge per validare le invarianti.

Conseguenze:

- Lettura e scrittura dello stesso artefatto passano per regole di parsing diverse.
  `Document::value()` fa `split_once(':')` riga per riga e non gestisce YAML annidato,
  liste multilinea o scalari a blocchi; il diagnostico del desktop (serde_yaml) li accetta.
  Le due viste possono **disaccordarsi sullo stesso file** — la classe di bug di
  BUG-001 / FIX-004 (status cartella vs frontmatter).
- `serde_yaml` è deprecato upstream (non più mantenuto).

**Direzione:** un solo punto di verità per parse/serialize del frontmatter, in
`lmbrain-core`, consumato sia dal desktop sia dall'MCP.

---

## Tema 2 — Duplicazione massiccia in `contract.rs`

[`src-tauri/src/commands/contract.rs`](src-tauri/src/commands/contract.rs) (999 righe)
contiene 7 funzioni `build_*` (`build_specs`, `build_reviews`, `build_adrs`,
`build_agents`, `build_mcp_records`, `build_mcp_proposals`, `build_handoffs`) che sono
**lo stesso scheletro copiato**: leggi dir → filtra `.md` → parse → estrai `id` con check
prefisso → estrai `title/status/created/updated/tags/links` → push.

~500 righe riducibili a ~150 con un helper generico parametrizzato sul tipo di artefatto.
Stessa storia, più in piccolo, per i `match status_str` ripetuti e per
`walk_md_files` / `scan` reimplementati in tre file diversi.

---

## Tema 3 — Stile incoerente in core e MCP

[`transitions.rs`](lmbrain-core/src/transitions.rs),
[`invariants.rs`](lmbrain-core/src/invariants.rs) e
[`lmbrain-mcp/src/main.rs`](lmbrain-mcp/src/main.rs) sono scritti come **one-liner
densissimi** (funzioni intere su una riga). Funzionano e sono testati, ma sono il punto
più difficile da mantenere del progetto e stonano col resto del codebase, ben formattato.
È il sottosistema più critico (muta file su disco) ed è il meno leggibile.

---

## Rilievi minori (concreti)

- **Bug reale — timestamp.** [`PathGuard::read_file`](src-tauri/src/commands/filesystem.rs)
  formatta il tempo come `"{days}d {hours}h {mins}m ago"` ma calcola i giorni
  **dall'epoch Unix** (1970), non "fa". Mostra valori tipo `20100d ago`.
- **`#![allow(dead_code)]` globale** in [`src-tauri/src/lib.rs`](src-tauri/src/lib.rs):
  maschera codice morto sull'intero crate. Es. `transition_from_proposed`
  ([transitions.rs](lmbrain-core/src/transitions.rs)) è ormai morto — il desktop usa
  `transition` diretto via `set_artifact_status`; `PathGuard::exists` / `clear_root`
  risultano non usati.
- **Due `PathGuard`** distinti ([filesystem.rs](src-tauri/src/commands/filesystem.rs)
  e `lmbrain-core/src/path.rs`) con responsabilità simili (validazione confini workspace).
- **Errori silenziati.** In `contract.rs` i `if let Ok(content) = read_to_string` scartano
  in silenzio i file illeggibili: un artefatto corrotto sparisce dalla UI senza diagnostica.
- **`AppError`** ([errors.rs](src-tauri/src/errors.rs)) appiattisce tutto in `String` verso
  il frontend; alcune varianti non sono mai costruite via `From` (es. `Git`). I lock dei
  `Mutex` usano `.unwrap()` ovunque: un panic in un thread avvelena il guard.
- **Frontend.** [`ProjectPulse.tsx`](src/components/Pulse/ProjectPulse.tsx) (1288 righe) ha
  styling inline pervasivo e **ricarica dati** che `WorkspaceContext` già carica (doppio
  fetch). [`WorkspaceContext`](src/context/WorkspaceContext.tsx) ha 20+ azioni `SET_*`
  quasi identiche collassabili in un `MERGE_DATA`.

---

## Piano fasato

### Fase 1 — Unificare il frontmatter *(priorità alta — rischio di correttezza)*

- Promuovere il parser di `lmbrain-core` a unico punto di verità: parse + accessor
  tipizzati (string / string-array / bool) + render, eliminando `serde_yaml`.
- `parser.rs` del desktop diventa un thin adapter sul core; `contract.rs` e gli
  invarianti leggono dalla stessa logica.
- Test di regressione su frontmatter "difficili": colon nei valori, liste, blocco
  `activity` multilinea, CRLF vs LF.

**Done quando:** un solo parser; nessun `serde_yaml` nel grafo; test verdi; nessuna
differenza di comportamento osservabile su un kit reale.

### Fase 2 — Deduplicare `contract.rs`

- Helper generico `build_artifacts<T>(dir, prefix, mapper)` + funzione comune per i campi
  condivisi (`title/status/created/updated/tags/links`).
- Centralizzare `walk_md_files` / `scan` in un'unica utility condivisa.

**Done quando:** `contract.rs` sotto ~500 righe; zero cambi di comportamento; test verdi.

### Fase 3 — Igiene Rust + bug

- Rimuovere `#![allow(dead_code)]`, eliminare il codice morto emerso, portare
  `cargo clippy --workspace` a zero warning.
- Correggere il bug del timestamp in `read_file` (vero "tempo trascorso").
- Rivedere `AppError` (varianti effettivamente usate, `From` coerenti) e i
  `Mutex::unwrap()` critici (gestione del poisoning).

**Done quando:** clippy pulito; `transition_from_proposed` e affini rimossi o riusati;
timestamp corretto in UI.

### Fase 4 — Leggibilità core + MCP

- Riformattare `transitions.rs`, `invariants.rs`, `main.rs` (MCP) in stile normale
  multi-riga, con i test esistenti a fare da rete. **Nessun cambio di logica.**
- Valutare l'unificazione dei due `PathGuard` sul core.

**Done quando:** i tre file sono leggibili come il resto del codebase; test invariati e verdi.

### Fase 5 — Frontend

- Snellire `WorkspaceContext` (azioni del reducer → `MERGE_DATA`).
- Togliere il doppio fetch in `ProjectPulse` (consumare lo stato dal context).
- Estrarre lo styling inline ripetuto in classi/CSS condivisi.

**Done quando:** nessun fetch duplicato; `ProjectPulse` significativamente più corto;
`pnpm test` + `pnpm lint` verdi.

---

## Note di processo

- Niente commit/push se non richiesto esplicitamente.
- Una fase alla volta, con diff e test verdi mostrati prima di proseguire.
- Verifica per fase: `cargo test --workspace` + `cargo clippy --workspace` + `pnpm test`
  (+ `pnpm lint` per la Fase 5).
