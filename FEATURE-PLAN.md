# LMBrain — Piano feature: view "Sessions" (lancio e monitoraggio di sessioni Claude Code)

> Stato: **proposto / decisioni bloccate**, non ancora implementato.
> Una view dedicata che lancia e monitora più sessioni interattive di Claude Code come
> finestre flottanti dentro l'app, tutte sul contesto della cartella del progetto lmbrain,
> con possibilità di lanciarle anche via `ollama launch claude --model <model>`.

## Decisioni prese (confermate con l'operatore)

1. **Terminale interattivo embedded.** PTY + `xterm.js` dentro l'app; si digita e si vede la
   TUI di Claude Code come in un vero terminale.
2. **Finestre flottanti in-app.** Un canvas dentro la view con finestre draggable / resizable /
   chiudibili (stile desktop), non finestre OS native, non tiling.
3. **Due modalità di lancio, modello a runtime.** `claude` nativo oppure
   `ollama launch claude --model <model>`, con scelta del modello al momento della creazione.
4. **Sessioni vive finché l'app è aperta.** Restano attive cambiando view; terminate alla
   chiusura dell'app. Nessuna riconnessione dopo riavvio in v1.
5. **Finestre via `react-rnd`** (drag + resize collaudati).
6. **Lista modelli in auto-discovery** (vedi sotto): niente lista hardcoded.

## Riscontri reali su ollama (verificati sulla macchina, ollama 0.30.10)

- `ollama launch claude --model <model>` è un comando reale (`launch` → integrazione `claude`)
  e si esegue **verbatim**, sostituendo solo il modello.
- I modelli disponibili **cambiano nel tempo** (l'operatore citava `glm-5.2:cloud`, sul
  sistema oggi c'è `glm-5.1:cloud`), quindi la lista va scoperta a runtime.
- Sorgente pulita: l'**API locale** `GET http://localhost:11434/api/tags` restituisce JSON con
  `name`, host remoto (per distinguere i `:cloud`) e soprattutto le **`capabilities`** di ogni
  modello (es. `["completion","tools","thinking"]`).
- **Filtro:** Claude Code richiede il tool-calling → mostrare solo i modelli con capability
  **`tools`** (così gli embed come `nomic-embed-text` spariscono da soli).
- La CSP della webview (`connect-src 'self' ipc:`) **blocca** `localhost:11434`: la chiamata
  deve passare dal backend Rust. Fallback a `ollama list` (parsing colonne) se l'API non risponde.

## Architettura

### Backend Rust — `SessionManager`
Service con una mappa `id → sessione`; ogni sessione possiede un PTY (`portable-pty`,
cross-platform, ConPTY su Windows) e il processo figlio.

Comandi Tauri:
- `session_start({ mode: "claude" | "ollama", model?: string, label?: string }) -> id`
  - `cwd` = root del progetto lmbrain (dal `PathGuard`), environment ereditato.
  - claude nativo → `claude`
  - ollama → `ollama launch claude --model <model>`
- `session_write(id, data)` — input dell'utente verso il PTY
- `session_resize(id, cols, rows)` — propaga il resize del terminale
- `session_kill(id)` — termina il processo
- `session_list() -> [{ id, label, mode, model, status }]` — riallinea la UI
- `list_ollama_models() -> [{ name, cloud, capabilities }]` — via `/api/tags`, filtrato `tools`

Eventi verso il frontend:
- `session-output { id, data }` — stream stdout/stderr del PTY
- `session-exit { id, code }`

Ciclo di vita: hook sull'uscita dell'app + `Drop` del service → kill di tutti i PTY (niente
processi orfani).

Permessi/sicurezza: i `#[tauri::command]` spawnano direttamente da Rust con `portable-pty`,
quindi **non serve il plugin shell**; la CSP attuale va bene (è il processo figlio a fare rete,
fuori dalla webview). L'esecuzione di processi amplia però la postura "read-only / desktop-first"
di [[ADR-001-desktop-first-tauri]] → va formalizzata con un **ADR dedicato** quando si parte.

### Frontend React — view "Sessions"
- Nuova chiave in `AppView` + voce sidebar (icona `terminal`) + routing in `AppShell`.
- **Canvas** con N **finestre flottanti** (`react-rnd`): drag, resize, pulsante chiudi,
  focus → z-index in cima.
- Ogni finestra incapsula un componente `Terminal` su `xterm.js` (+ `@xterm/addon-fit`):
  si iscrive a `session-output` → `term.write`; `term.onData` → `session_write`;
  su resize → `session_resize`.
- **Toolbar / modale "Nuova sessione":** modalità (claude / ollama); se ollama, dropdown
  modelli (auto-discovery + refresh); label.
- **Persistenza finché l'app è aperta:** la `SessionsView` resta **sempre montata** (nascosta
  via CSS quando non è la view attiva), così le istanze xterm e lo scrollback sopravvivono al
  cambio view senza bufferizzare l'output lato Rust. Le geometrie delle finestre vivono in
  `WorkspaceContext`.

### Settings (estensione)
- Path opzionali dei binari `claude` / `ollama` (default: PATH).
- (Eventuale) toggle per mostrare anche i modelli non-`tools`.

## Fasi (incrementali, ognuna verificabile)

1. **Backend PTY headless** — `SessionManager` + comandi/eventi + kill-on-exit, testato con un
   comando fittizio (es. una shell) prima di toccare claude/ollama.
2. **Componente Terminal** — un singolo xterm embedded agganciato a una sessione (niente finestre
   ancora). *Qui si verifica presto il rendering della TUI di Claude Code in xterm.*
3. **Window manager** — multi-sessione, finestre flottanti `react-rnd` drag/resize/close + focus.
4. **Launch modal + Ollama** — scelta modalità/modello, `list_ollama_models`, integrazione
   `ollama launch`.
5. **Rifinitura + release** — indicatori di stato (running / exited / exit-code), ADR,
   CHANGELOG, bump versione.

## Nuove dipendenze

- Rust: `portable-pty`; un mini client HTTP per il localhost (`ureq`, leggero, no TLS).
- Frontend: `xterm` + `@xterm/addon-fit`; `react-rnd`.

## Rischi noti

- **Rendering TUI di Claude Code in xterm** (alt-screen, mouse, 256 colori): di norma ok, ma da
  verificare già in Fase 2 — è il punto che può sorprendere.
- **ConPTY su Windows**: qualche spigolo su resize/segnali.
- **Risorse**: ogni sessione è processo + xterm; con molte sessioni la view pesa → eventuale cap
  morbido configurabile.
- **Ambiente del processo figlio**: si assume `claude` già loggato e `ollama` già autenticato
  (confermato dall'operatore); l'app eredita l'environment, non gestisce login.

## Out of scope (v1)

- Riconnessione delle sessioni dopo riavvio dell'app.
- Iniezione di profili agente / system prompt nelle sessioni.
- cwd selezionabile per finestra (fisso alla root lmbrain).
