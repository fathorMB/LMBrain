# LMBrain — Bug Report

> Documento di tracciamento bug rilevati durante il testing manuale dell'app.
> Da condividere con il team di sviluppo per la risoluzione.

- **App:** LMBrain
- **Versione testata:** 1.1.1
- **Tester:** Moreno Bruschi
- **Inizio sessione di test:** 2026-06-23
- **Ambiente:** Windows 11 Pro (10.0.22631) — build desktop Tauri

---

## Legenda

**Severità**
- 🔴 **Critica** — crash, perdita dati, funzione core inutilizzabile, blocco totale
- 🟠 **Alta** — funzione importante non utilizzabile, nessun workaround
- 🟡 **Media** — malfunzionamento con workaround possibile
- 🔵 **Bassa** — difetto minore, estetico, cosmetico

**Stato**
- `Aperto` — segnalato, non ancora preso in carico
- `In corso` — in lavorazione dai dev
- `Da verificare` — fixato, in attesa di retest
- `Chiuso` — verificato e risolto
- `Non riproducibile` / `Won't fix` — secondo decisione dei dev

---

## Riepilogo

| ID | Titolo | Area | Severità | Stato |
|------|--------|------|----------|-------|
| [BUG-001](#bug-001--lo-stato-dei-task-nella-board-dipende-solo-dalla-cartella-il-campo-status-del-frontmatter-viene-ignorato) | Lo stato dei task nella board dipende solo dalla cartella; il campo `status:` del frontmatter viene ignorato | Taskboard | 🟠 Alta | Risolto in sessione (FIX-004; badge FIX-003). Modifica backend da validare in CI |
| [BUG-002](#bug-002--lapp-non-segnala-quando-recommended_agent-di-una-spec-non-punta-a-un-profilo-agente-esistente) | L'app non segnala quando `recommended_agent` di una spec non punta a un profilo agente esistente | Diagnostics / Project Pulse | 🟡 Media | Risolto in sessione · CI da validare |
| [BUG-003](#bug-003--il-ciclo-di-vita-dei-task-non-è-supportato-default-planned-e-nessuna-transizione-in-progress-allavvio) | Il ciclo di vita dei task non è supportato: default `planned` e nessuna transizione a `in-progress` all'avvio | Kit (template/AGENT/handoff) | 🟠 Alta | Risolto in sessione · CI da validare |
| [BUG-004](#bug-004--il-lead-dopo-lapprovazione-delle-adr-si-mette-a-implementare-lo-scaffolding) | Il lead, dopo l'approvazione delle ADR, si mette a implementare lo scaffolding | Kit (AGENT.md / bootstrap prompt) | 🟠 Alta | Risolto in sessione |
| [BUG-005](#bug-005--il-prompt-di-handoff-genera-un-path-spec-senza-slug-che-non-esiste) | Il prompt di handoff genera un path spec senza slug (`SPEC-017.md`) che non esiste | Project Pulse / handoffPrompt | 🟡 Media | Risolto in sessione |
| [BUG-006](#bug-006--lmcp-non-è-realmente-registrato-nellhost-gli-agenti-non-hanno-i-tool-e-editano-a-mano) | L'MCP non è realmente registrato nell'host: gli agenti non hanno i tool e editano a mano | Kit (bootstrap MCP registration) / distribuzione | 🟠 Alta | Aperto |
| [FIX-001](#fix-001--button-copy-prompt--hide-prompt-fuori-stile-in-next-recommended-actions) | Button "Copy prompt" / "Hide prompt" fuori stile in Next Recommended Actions | Project Pulse | 🔵 Bassa | Risolto in sessione |
| [FIX-002](#fix-002--markup-bold-e-wikilinks-mostrati-grezzi-nel-project-pulse-invece-di-essere-formattatilink) | Markup `**bold**` e `[[wikilink]]` mostrati grezzi nel Project Pulse invece di essere formattati/link | Project Pulse | 🟡 Media | Risolto in sessione |
| [FIX-003](#fix-003--mismatch-cartellafrontmatter-dello-stato-task-reso-visibile-sulla-card-mitigazione-di-bug-001) | Mismatch cartella/frontmatter dello stato task reso visibile sulla card (mitigazione di BUG-001) | Taskboard | 🟠 Alta | Risolto in sessione |
| [FIX-004](#fix-004--la-colonna-del-task-segue-il-campo-status-del-frontmatter-il-cambio-stato-muove-la-card) | La colonna del task segue il campo `status:` del frontmatter: il cambio stato muove la card | Taskboard (backend) | 🟠 Alta | Risolto in sessione · CI da validare |
| [FIX-005](#fix-005--wikilink-grezzi-non-cliccabili-anche-in-roadmap-e-nei-blockerazioni-del-pulse) | Wikilink grezzi/non cliccabili anche in Roadmap e nei blocker/azioni del Pulse | Roadmap / Project Pulse | 🟡 Media | Risolto in sessione |

---

## Dettaglio bug

### BUG-001 — Lo stato dei task nella board dipende solo dalla cartella; il campo `status:` del frontmatter viene ignorato

- **Area / Schermata:** Taskboard
- **Severità:** 🟠 Alta
- **Stato:** Risolto in sessione — [FIX-004](#fix-004--la-colonna-del-task-segue-il-campo-status-del-frontmatter-il-cambio-stato-muove-la-card) (la card segue il `status:` del frontmatter) + [FIX-003](#fix-003--mismatch-cartellafrontmatter-dello-stato-task-reso-visibile-sulla-card-mitigazione-di-bug-001) (badge di divergenza). Modifica backend Rust da validare in CI
- **Data rilevamento:** 2026-06-23
- **Versione:** 1.1.1

**Descrizione**
Nella board i task non cambiano mai colonna di stato. Il ciclo di vita atteso
(`backlog` → `planned` → `in-progress` → `review` → `done`) è guidato dal role
**project-lead/manager**, non dall'operator. Il problema è che la board determina
lo stato di un task **esclusivamente dalla cartella** in cui si trova il file
`.md` (`.lmbrain/tasks/<stato>/`), ignorando completamente il campo `status:` del
frontmatter. Di conseguenza, se una transizione di stato aggiorna il campo
`status:` ma **non sposta fisicamente il file** nella cartella corrispondente (o
viceversa), la card non si muove mai e l'app non offre alcun modo per riconciliare
le due fonti.

**Causa tecnica (analisi codice)**
- `build_tasks` assegna lo stato dal nome della directory, non dal frontmatter:
  lo stato è `status: status.clone()` dove `status` è la cartella iterata.
  → [`src-tauri/src/commands/contract.rs:27-73`](src-tauri/src/commands/contract.rs:27)
- Il campo `status:` del frontmatter viene letto **solo** per generare un
  *diagnostic* di mismatch cartella/stato (vedi `CONTRACT.md` invariante
  "directory/status mismatch"), mai per posizionare la card.
- La board e il drawer sono **read-only** e non offrono alcun controllo per
  cambiare stato: *"Read-only view · status changes happen in Markdown"*.
  → [`src/components/Taskboard/TaskboardView.tsx:95`](src/components/Taskboard/TaskboardView.tsx:95),
    [`src/components/Taskboard/TaskDrawer.tsx`](src/components/Taskboard/TaskDrawer.tsx)
- L'unico comando di scrittura stato in-app, `set_artifact_status`, gestisce solo
  SPEC/ADR/AGENT/MCP, **riscrive solo il frontmatter senza spostare il file**, e
  **non contempla affatto i task** (richiede `status == "proposed"` di partenza).
  → [`src-tauri/src/commands/contract.rs:1094-1190`](src-tauri/src/commands/contract.rs:1094)
- Il contratto richiede coerenza tra le due fonti — *"The filesystem and `status`
  frontmatter must agree where a status directory exists"* — ma la coerenza è
  demandata a chi scrive i file; l'app non la garantisce né la corregge.
  → [`.lmbrain/CONTRACT.md:13`](.lmbrain/CONTRACT.md:13),
    [`.lmbrain/tasks/README.md:3`](.lmbrain/tasks/README.md:3)

**Passi per riprodurre**
1. Aprire un workspace con dei task (es. quelli in `.lmbrain/tasks/review/`).
2. Far avanzare di stato un task aggiornando **solo** il campo `status:` nel
   frontmatter del file `.md`, senza spostarlo di cartella (oppure spostarlo di
   cartella senza aggiornare il campo).
3. Osservare la Taskboard.

**Comportamento atteso**
La card del task si sposta nella colonna corrispondente al nuovo stato dichiarato,
coerentemente con il fatto che il frontmatter è descritto come "source of truth"
dei metadati e che la transizione è un'operazione di prima classe del workflow.

**Comportamento osservato**
La card resta nella colonna corrispondente alla **cartella** del file. Una modifica
del solo campo `status:` non ha alcun effetto sulla board (genera al più un
diagnostic di mismatch). Non esiste alcun controllo nell'app per cambiare stato a
un task.

**Frequenza:** sempre (deterministico)

**Note tecniche / impatto**
- Doppia fonte di verità fragile (cartella + frontmatter) senza riconciliazione
  applicativa: è facile finire in stato incoerente in modo silenzioso.
- Per i dev — possibili direzioni di fix da valutare:
  1. far derivare lo stato della board dal campo `status:` del frontmatter (fonte
     unica), oppure
  2. quando cartella e `status:` divergono, mostrare esplicitamente il conflitto
     sulla card (oltre al diagnostic), oppure
  3. introdurre un'azione in-app di transizione task che sposti il file **e**
     aggiorni il frontmatter in modo atomico (come `set_artifact_status` fa per gli
     altri artefatti, estesa ai TASK e al move di cartella).
- Nota: quando il file viene **spostato di cartella** in modo corretto, il file
  watcher rileva l'evento (Remove+Create su `.md`, debounce 500ms) ed emette
  `file-changed`, quindi la board si ricarica. Il difetto riguarda specificamente
  l'allineamento cartella/frontmatter e l'assenza di una transizione gestita.
  → [`src-tauri/src/commands/watcher.rs:71-124`](src-tauri/src/commands/watcher.rs:71)

**Da confermare con i dev:** qual è la fonte di verità voluta per lo stato del task
— la cartella o il campo `status:`? Il fix dipende da questa decisione di design.

---

### BUG-002 — L'app non segnala quando `recommended_agent` di una spec non punta a un profilo agente esistente

- **Area / Schermata:** Diagnostics / Project Pulse ("Next Recommended Actions")
- **Severità:** 🟡 Media
- **Stato:** Risolto in sessione (diagnostic aggiunto) · **backend Rust da validare in CI**
- **Data rilevamento:** 2026-06-23
- **Versione:** 1.2.1

**Descrizione**
Una spec può dichiarare `recommended_agent:` con un valore che non corrisponde ad
alcun profilo di agente esistente (es. il placeholder del template `AGENT-XXX`, o un
ID errato). L'app non lo segnala in alcun modo: nel Pulse mostra letteralmente
`Start AGENT-XXX on SPEC-001`, e il prompt di handoff generato istruisce l'agente a
impersonare un profilo inesistente.

**Come è emerso**
Testando il progetto di prova `E:\Git\Brewlog`: la sua `SPEC-001` ha
`recommended_agent: AGENT-XXX` (placeholder non sostituito dal project-lead) e in
`agents/profiles/` esiste solo `project-lead.md`. La card "Next Recommended Actions"
mostra `Start AGENT-XXX on SPEC-001`. (Nota: la causa diretta è un dato incompleto in
Brewlog, ma questo bug riguarda il fatto che **LMBrain non lo intercetta**.)

**Comportamento atteso**
Coerentemente con `CONTRACT.md` — *"The application should warn about duplicate IDs,
broken links, directory/status mismatches, missing references, and circular
dependencies"* — un `recommended_agent` che non risolve a un profilo agente
esistente (o ancora al placeholder `AGENT-XXX`) dovrebbe produrre un **diagnostic di
warning** (missing reference).

**Comportamento osservato**
Nessun diagnostic. Il valore viene propagato tale e quale nella UI e nel prompt.

**Causa tecnica**
`build_diagnostics` non valida il campo `recommended_agent` contro i profili in
`agents/profiles/`. Il valore è solo letto e mostrato.
→ [`src-tauri/src/commands/contract.rs:139`](src-tauri/src/commands/contract.rs:139),
  [`src-tauri/src/commands/contract.rs:684`](src-tauri/src/commands/contract.rs:684)

**Fix applicata**
`build_diagnostics` ora valida ogni spec con `recommended_agent` valorizzato:
emette un warning *"Missing reference: spec … recommends agent '…', which is not an
existing agent profile"* se il valore è un placeholder (`*-XXX`) o non corrisponde
all'`id` di un profilo in `agents/profiles/`. Il warning appare nella sezione
Diagnostics del Pulse (con relativo fix-prompt).
- → [`src-tauri/src/commands/contract.rs`](src-tauri/src/commands/contract.rs) (in `build_diagnostics`)
- Aggiunti due test (placeholder non risolto → warning; agente esistente → nessun warning).
  → [`src-tauri/tests/contract_test.rs`](src-tauri/tests/contract_test.rs)

**⚠️ Verifica:** modifica backend Rust, **non compilabile in locale** → da validare
in **CI** (`cargo test`). Nessuna modifica frontend necessaria (il Pulse mostra già i
diagnostics).

---

### BUG-003 — Il ciclo di vita dei task non è supportato: default `planned` e nessuna transizione a `in-progress` all'avvio

- **Area / Schermata:** Kit (template task, `AGENT.md`, prompt di handoff) + eventuale diagnostic app
- **Severità:** 🟠 Alta
- **Stato:** Risolto in sessione (tutti e 4 gli interventi) · diagnostic backend da validare in CI
- **Data rilevamento:** 2026-06-23
- **Versione:** 1.2.4

**Ciclo di vita atteso (dall'operatore)**
`backlog` (task emerso, spec non ancora pronta) → `planned` (il lead prepara la
spec) → `in-progress` (l'esecutore inizia) → `review` (l'esecutore finisce) →
`done` (il reviewer accetta).

**Comportamento osservato**
1. Quando il dev-agent inizia a lavorare, il task **non passa a `in-progress`**.
2. I task risultano `planned` anche **senza una spec attiva** (dovrebbero essere
   `backlog` e diventare `planned` solo quando il lead prepara la spec).

**Causa tecnica (lacune di convenzione del kit)**
- Il template task ha default `status: planned`, quindi ogni task nasce `planned`
  a prescindere dallo stato della spec.
  → [`kit/.lmbrain/templates/task.md:5`](kit/.lmbrain/templates/task.md:5)
- `AGENT.md` non documenta il ciclo di vita né l'obbligo per l'esecutore di portare
  il task a `in-progress` all'avvio e a `review` a fine lavoro.
  → [`kit/.lmbrain/AGENT.md`](kit/.lmbrain/AGENT.md)
- Il prompt di handoff generato non menziona alcuna transizione di stato del task.
  → [`src/lib/handoffPrompt.ts`](src/lib/handoffPrompt.ts)
- Conferma sui dati Brewlog: `TASK-002` ha `spec:` vuoto ma è in `planned`.

**Interventi applicati (tutti e 4, scelti dall'operatore)**
1. **Template task → `backlog`** (kit + live): un task nasce in backlog, non planned.
   → [`kit/.lmbrain/templates/task.md`](kit/.lmbrain/templates/task.md), [`.lmbrain/templates/task.md`](.lmbrain/templates/task.md)
2. **Prompt di handoff**: ora istruisce l'esecutore a portare il/i task `in-progress`
   prima di iniziare e `review` a fine lavoro.
   → [`src/lib/handoffPrompt.ts`](src/lib/handoffPrompt.ts)
3. **Diagnostic** (backend Rust): warning quando un task è `planned` ma non ha una
   spec collegata pronta (spec mancante, inesistente, o non `ready`/`in-progress`/
   `review`/`accepted`). Appare nei Diagnostics del Pulse. Aggiunti 2 test.
   → [`src-tauri/src/commands/contract.rs`](src-tauri/src/commands/contract.rs),
     [`src-tauri/tests/contract_test.rs`](src-tauri/tests/contract_test.rs)
4. **Documentazione del lifecycle** in `AGENT.md` e `tasks/README.md` (kit + live):
   `backlog → planned → in-progress → review → done` con i rispettivi owner.

**⚠️ Verifica:** frontend 43/43, `tsc`/`eslint` puliti. Il diagnostic Rust si valida
in CI (`cargo test`). Nota: i dati esistenti di Brewlog (es. `TASK-002` planned senza
spec) non sono toccati — il nuovo diagnostic te li segnalerà.

---

### BUG-004 — Il lead, dopo l'approvazione delle ADR, si mette a implementare lo scaffolding

- **Area / Schermata:** Kit (`AGENT.md`, prompt di bootstrap del Project Lead)
- **Severità:** 🟠 Alta
- **Stato:** Risolto in sessione (rinforzo guardrail)
- **Data rilevamento:** 2026-06-23
- **Versione:** 1.2.5

**Descrizione**
Più volte il Project Lead, dopo che l'operatore ha approvato le ADR (es. lo stack
tecnologico), ha iniziato a **implementare lo scaffolding** del progetto. Il lead non
deve implementare: deve produrre l'handoff a uno specialista. Lo scaffolding/setup
iniziale è lavoro di implementazione.

**Causa (lacuna di guardrail, non un bug app)**
Il confine in `AGENT.md` vietava già di modificare codice/build/config, ma non
chiariva due cose che il lead razionalizzava:
1. che **scaffolding/setup/bootstrapping sono implementazione** (il lead poteva
   pensare "non sono *feature*, quindi posso");
2. che **approvare ADR/spec/stack non autorizza a implementare** (il lead leggeva
   l'approvazione come "via libera").
→ [`kit/.lmbrain/AGENT.md`](kit/.lmbrain/AGENT.md), [`kit/.lmbrain/templates/project-lead-bootstrap-prompt.md`](kit/.lmbrain/templates/project-lead-bootstrap-prompt.md)

**Fix applicata (kit + live)**
- In `AGENT.md`, aggiunto un paragrafo esplicito: il confine copre **scaffolding,
  setup, installazione dipendenze e bootstrapping**; **approvare una ADR/spec/
  direzione tecnica non autorizza mai a implementare**; dopo l'approvazione l'unico
  passo è preparare l'handoff (spec path + specialista) e fermarsi; se non esiste un
  profilo specialista, lo propone e attende.
- Nel prompt di bootstrap, aggiunta la stessa precisazione ("include scaffolding e
  setup; approvare lo stack/ADR/spec non autorizza a implementare: prepara l'handoff
  e fermati").

**Nota:** è un rinforzo di una regola già esistente (nessun cambiamento di design),
solo documentazione/prompt del kit — nessuna modifica di codice. L'enforcement vero
resta comportamentale (l'app non può impedirlo).

---

### BUG-005 — Il prompt di handoff genera un path spec senza slug che non esiste

- **Area / Schermata:** Project Pulse ("Next Recommended Actions") / `buildHandoffPrompt`
- **Severità:** 🟡 Media
- **Stato:** Aperto — da fixare in fase di review di SPEC-017 (il file è toccato anche dall'implementatore)
- **Data rilevamento:** 2026-06-23
- **Versione:** 1.2.6

**Descrizione**
Il prompt di handoff generato costruisce il path della spec come
`.lmbrain/specs/<status>/<SPEC-ID>.md` (es. `.lmbrain/specs/ready/SPEC-017.md`), ma i
file reali hanno lo **slug** nel nome (es. `SPEC-017-controlled-mutation-engine.md`).
Il path generato quindi **non esiste**, e un agente che lo apre fallisce.

**Causa tecnica**
`buildHandoffPrompt(agent, specId, status)` interpola `${specId}.md` invece di
ricavare il nome-file reale (per id) dall'elenco delle spec.
→ [`src/lib/handoffPrompt.ts`](src/lib/handoffPrompt.ts)

**Fix applicata**
`buildHandoffPrompt` accetta ora il nome-file reale della spec; `ActionCard` lo
risolve da `state.specs` (per id → basename del `path`), con fallback a `SPEC-ID.md`
solo se la spec non è caricata. Preservata la guida MCP aggiunta dall'implementatore.
→ [`src/lib/handoffPrompt.ts`](src/lib/handoffPrompt.ts), [`src/components/Pulse/ProjectPulse.tsx`](src/components/Pulse/ProjectPulse.tsx)

**Verifica:** `tsc` pulito; `ProjectPulse.test` 3/3 (il fallback mantiene verde il test).

---

### BUG-006 — L'MCP non è realmente registrato nell'host: gli agenti non hanno i tool e editano a mano

- **Area / Schermata:** Kit (registrazione MCP nel bootstrap) + distribuzione del binario
- **Severità:** 🟠 Alta
- **Stato:** Aperto
- **Data rilevamento:** 2026-06-23
- **Versione:** 1.3.1

**Descrizione**
Gli agenti (es. Claude Code come Project Lead) **non ricevono i tool `lmbrain-mcp`**:
il server non è effettivamente registrato nell'host. Di conseguenza il lead ripiega
sull'editing manuale dei markdown, producendo stati incoerenti.

**Come è emerso (Brewlog)**
Dopo la rigenerazione di Brewlog col lead: `SPEC-001` ha `status: ready` nel
frontmatter ma il file è in `specs/proposed/` (mismatch — il campo è stato cambiato a
mano invece di usare `spec_ready`, che avrebbe spostato il file); nessun task creato
(`tasks/*/` contiene solo `.gitkeep`); `recommended_agent: AGENT-001` non esiste. La
taskboard è vuota perché non ci sono task.

**Causa tecnica**
- Brewlog non ha alcun `.mcp.json` alla radice (il file che Claude Code legge per i
  server MCP di progetto).
- Il template del kit `kit/.lmbrain/mcp/lmbrain-mcp.json` usa un formato **custom**
  (`{name, transport, command, cwd, scope, …}`) che nessun host MCP interpreta;
  Claude Code richiede `{"mcpServers": {"<name>": {"command": …, "args": []}}}`.
- **Non esiste codice che registri il server** (nessuna scrittura di `.mcp.json` né
  configurazione dell'host): il criterio di SPEC-017 *"the kit bootstrap registers
  the MCP server"* è dichiarato ma non implementato.
- Il binario `lmbrain-mcp` è solo un artefatto CI: **non è distribuito/installato**
  sulla macchina né in PATH, quindi anche un `.mcp.json` corretto non risolverebbe il
  comando.

**Nota di processo**
La review REVIEW-014 ha **mancato** questo punto: la CI valida engine + build del
binario ma non il cablaggio end-to-end con l'host (lo smoke-test "an agent host
connects the MCP server" non è mai stato eseguito). L'accettazione avrebbe dovuto
escludere questo criterio.

**Cosa serve (proposta)**
1. **Registrazione reale**: durante il bootstrap/init, scrivere un `.mcp.json` nel
   formato dell'host (Claude Code) alla radice del repo, con `command` verso il
   binario installato.
2. **Distribuzione**: installare/risolvere `lmbrain-mcp` (in PATH o percorso noto;
   es. bundle con l'app o `cargo install`), così il `command` funziona.
3. Aggiornare il template del kit al formato corretto (o rimuoverlo a favore della
   scrittura programmatica).

---

## Fix applicate durante il test

> Interventi cosmetici/minori risolti direttamente in sessione (non in attesa dei
> dev). Tracciati qui per completezza e per il retest visivo.

### FIX-001 — Button "Copy prompt" / "Hide prompt" fuori stile in Next Recommended Actions

- **Area / Schermata:** Project Pulse → "Next Recommended Actions"
- **Severità:** 🔵 Bassa (cosmetico)
- **Stato:** Risolto in sessione (da verificare a video)
- **Data rilevamento:** 2026-06-23
- **Versione:** 1.1.1

**Descrizione**
I due button "Copy prompt" e "Hide prompt" / "View prompt" della card di azione
consigliata erano renderizzati con lo stile di default del browser (sfondo bianco),
fuori dal tema scuro dell'app.

**Causa tecnica**
I due `<button>` in `RecommendedActionRow` non avevano alcuno stile inline, quindi
ereditavano lo stile di default dello user agent. Anche i messaggi di feedback
(`Copied to clipboard.` / errore) erano testo bianco grezzo.
→ [`src/components/Pulse/ProjectPulse.tsx`](src/components/Pulse/ProjectPulse.tsx)

**Fix applicata**
Uniformati ai pattern di button già presenti nello stesso componente:
- "Copy prompt" → stile **primario pieno** (`var(--accent-light)` + testo bianco +
  icona `content_copy`), coerente con "Copy fix prompt".
- "View / Hide prompt" → stile **secondario sottile** (`rgba(255,255,255,0.06)` con
  bordo leggero + icona `visibility`/`visibility_off`), coerente con il button "Fix".
- Messaggi di feedback colorati a tema (verde per il successo, rosso per l'errore).
- Testi e attributi ARIA invariati (`role="status"`, `aria-expanded`).

**Verifica:** `pnpm vitest run src/__tests__/ProjectPulse.test.tsx` → 3/3 passati.
Resta da confermare visivamente a video.

---

### FIX-002 — Markup `**bold**` e `[[wikilink]]` mostrati grezzi nel Project Pulse invece di essere formattati/link

- **Area / Schermata:** Project Pulse (breadcrumb milestone, "Current focus", card milestone)
- **Severità:** 🟡 Media
- **Stato:** Risolto in sessione (da verificare a video)
- **Data rilevamento:** 2026-06-23
- **Versione:** 1.1.1

**Descrizione**
Nel Project Pulse alcune stringhe mostravano il markup Markdown grezzo invece di
renderizzarlo: il grassetto `**...**` appariva con gli asterischi e i riferimenti
`[[ROADMAP]]` / `[[SPEC-001]]` come testo letterale tra doppie quadre, anziché come
link cliccabili. Riguardava: il breadcrumb sopra "Project Pulse", la frase
"Current focus:" e il titolo della card milestone.

**Causa tecnica**
I campi `pulse.milestone` e `pulse.focus` venivano stampati come testo semplice
(`{pulse.milestone}` / `{pulse.focus}`) senza passare per alcun renderer. L'app ha
un `MarkdownRenderer` completo (con gestione dei `[[wikilink]]`) ma è un renderer a
blocchi, inadatto a questi contesti inline.
→ [`src/components/Pulse/ProjectPulse.tsx`](src/components/Pulse/ProjectPulse.tsx),
  [`src/lib/markdown.tsx`](src/lib/markdown.tsx)

**Fix applicata**
- Aggiunto un renderer inline leggero e riutilizzabile, `InlineRichText`, che
  interpreta `**bold**` e `[[target]]` / `[[target|label]]` senza wrappare il
  contenuto in elementi a blocco.
  → [`src/lib/inlineRichText.tsx`](src/lib/inlineRichText.tsx)
- I wikilink usano lo stesso stile visivo dei wikilink dell'app (accent +
  sottolineatura tratteggiata) e al click navigano alla sezione **Wiki** aprendo il
  documento target (risoluzione via `resolveWikilink` sul wiki tree, caricato in
  modo lazy solo al click per non appesantire il load iniziale della pulse).
- `WikiView` ora rispetta la pagina selezionata da fuori (deep-link), così il click
  dalla pulse apre davvero il documento.
  → [`src/components/Wiki/WikiView.tsx`](src/components/Wiki/WikiView.tsx)

**Limitazione nota (per i dev)**
La risoluzione del wikilink fa match per nome-file. I riferimenti "stretti" come
`[[SPEC-001]]` non combaciano con il nome file completo (es.
`SPEC-001-tauri-read-only-desktop-mvp.md`), quindi al momento un click su
`[[SPEC-001]]` apre la sezione Wiki senza puntare al documento specifico. `[[ROADMAP]]`
e simili (match esatto col nome file) si aprono correttamente. Valutare una
risoluzione per prefisso-ID lato wiki.

**Verifica:** `pnpm vitest run` → 41/41 passati; `tsc --noEmit` e `eslint` puliti
sui file toccati. Resta da confermare visivamente a video.

---

### FIX-003 — Mismatch cartella/frontmatter dello stato task reso visibile sulla card (mitigazione di BUG-001)

- **Area / Schermata:** Taskboard
- **Severità:** 🟠 Alta
- **Stato:** Risolto in sessione (mitigazione di [BUG-001](#bug-001--lo-stato-dei-task-nella-board-dipende-solo-dalla-cartella-il-campo-status-del-frontmatter-viene-ignorato); da verificare a video)
- **Data rilevamento:** 2026-06-23
- **Versione:** 1.1.1

**Decisione di design** (poi aggiornata)
Inizialmente si era scelto di mantenere la **cartella** come fonte di verità,
limitandosi a rendere visibile la divergenza. Su indicazione dell'operatore la
decisione è stata cambiata: **il `status:` del frontmatter diventa la fonte di
verità** per la colonna, così il cambio di stato muove davvero la card (vedi
[FIX-004](#fix-004--la-colonna-del-task-segue-il-campo-status-del-frontmatter-il-cambio-stato-muove-la-card)).
Questo badge resta comunque utile come **segnale complementare**: indica che il file
va ancora spostato nella cartella corrispondente per riallineare il filesystem.

**Cosa è stato fatto**
Il vero difetto applicativo non era il modello, ma il fatto che una divergenza
cartella/frontmatter fosse **silenziosa** sulla board (segnalata solo come diagnostic
generico altrove), facendo sembrare il task "fermo". Ora la Taskboard recupera i
diagnostics, individua i warning "Status mismatch" e li abbina ai task per path,
mostrando sulla card un avviso ambra: `status: <frontmatter> ≠ folder: <cartella>`
(con il messaggio completo nel tooltip). Così una transizione applicata a metà
(campo `status:` aggiornato ma file non spostato, o viceversa) è subito evidente.
- Intervento **solo frontend**, riusa la rilevazione mismatch già esistente nel
  backend → nessuna modifica Rust (non compilabile in locale).
  → [`src/components/Taskboard/TaskboardView.tsx`](src/components/Taskboard/TaskboardView.tsx)
- Aggiunto test di copertura.
  → [`src/__tests__/TaskboardView.test.tsx`](src/__tests__/TaskboardView.test.tsx)

**Cosa resta aperto (per i dev)**
La domanda di design di fondo di BUG-001 resta: se in futuro si vuole che un task
"si muova" agendo sul solo campo `status:`, serve decidere se rendere il frontmatter
la fonte di verità (con impatto su spec/review) oppure aggiungere una transizione
atomica che sposti il file e aggiorni il frontmatter. La mitigazione attuale non
cambia il modello, lo rende solo trasparente.

**Verifica:** `pnpm vitest run` → 43/43 passati; `tsc --noEmit` e `eslint` puliti
sui file toccati. Resta da confermare visivamente a video.

---

### FIX-004 — La colonna del task segue il campo `status:` del frontmatter: il cambio stato muove la card

- **Area / Schermata:** Taskboard (backend Rust)
- **Severità:** 🟠 Alta
- **Stato:** Risolto in sessione — **compilazione/test Rust da validare in CI** (toolchain non disponibile in locale)
- **Data rilevamento:** 2026-06-23
- **Versione:** 1.1.1

**Descrizione**
Risolve la radice di [BUG-001](#bug-001--lo-stato-dei-task-nella-board-dipende-solo-dalla-cartella-il-campo-status-del-frontmatter-viene-ignorato):
ora un cambio di stato si riflette nel movimento della card nella colonna giusta.

**Decisione di design**
Su indicazione dell'operatore, **il campo `status:` del frontmatter è la fonte di
verità** per la colonna della board. La cartella deve restare allineata, ma non
determina più da sola la posizione della card. Se il frontmatter manca o ha un
valore non valido, si ricade sullo stato derivato dalla cartella (compatibilità).

**Cosa è stato fatto (backend)**
- `build_tasks` ora deriva lo stato del task dal `status:` del frontmatter (con
  fallback alla cartella), invece di usare sempre il nome della cartella.
  → [`src-tauri/src/commands/contract.rs`](src-tauri/src/commands/contract.rs)
- Aggiunto `TaskStatus::from_str` per il parsing dello stato dichiarato.
  → [`src-tauri/src/models/task.rs`](src-tauri/src/models/task.rs)
- Aggiunti due test d'integrazione: lo stato segue il frontmatter; fallback alla
  cartella quando il frontmatter è assente.
  → [`src-tauri/tests/contract_test.rs`](src-tauri/tests/contract_test.rs)

**Effetti collaterali (positivi)**
Anche le metriche del Project Pulse che contano i task per stato (in-progress,
review, blocked) ora seguono il frontmatter, coerentemente con la board.

**⚠️ Verifica**
La modifica è **frontend-free** ma tocca il backend Rust, che **non è compilabile
in locale** (nessun cargo/rustc): va validata in **CI** (`cargo test`). Il badge di
divergenza di FIX-003 continua a funzionare come segnale per riallineare il file su
disco.

---

### FIX-005 — Wikilink grezzi/non cliccabili anche in Roadmap e nei blocker/azioni del Pulse

- **Area / Schermata:** Roadmap, Project Pulse (blocker e azioni consigliate)
- **Severità:** 🟡 Media
- **Stato:** Risolto in sessione (da verificare a video)
- **Data rilevamento:** 2026-06-23
- **Versione:** 1.2.0

**Descrizione**
Estende [FIX-002](#fix-002--markup-bold-e-wikilinks-mostrati-grezzi-nel-project-pulse-invece-di-essere-formattatilink):
i `[[wikilink]]` comparivano ancora grezzi (con le doppie quadre) e non cliccabili
in altri campi di testo libero — il titolo e l'outcome delle milestone nella
**Roadmap**, e titolo/descrizione di **blocker** e **azioni consigliate** nel Pulse.

**Causa tecnica**
Quei campi venivano stampati come testo semplice (`{m.outcome}`, `{action.description}`,
ecc.) senza passare per il renderer inline introdotto in FIX-002.
→ [`src/components/Roadmap/RoadmapView.tsx`](src/components/Roadmap/RoadmapView.tsx),
  [`src/components/Pulse/ProjectPulse.tsx`](src/components/Pulse/ProjectPulse.tsx)

**Fix applicata**
- Estratta la logica di navigazione del wikilink in un hook riutilizzabile
  `useWikiNavigation`, per non duplicarla tra le viste.
  → [`src/hooks/useWikiNavigation.ts`](src/hooks/useWikiNavigation.ts)
- Applicato `InlineRichText` a `m.title`/`m.outcome` (Roadmap) e a
  `title`/`description` di blocker e azioni (Pulse). I wikilink ora sono cliccabili
  e aprono il target nella sezione Wiki.

**Verifica:** `pnpm vitest run` → 43/43; `tsc --noEmit` ed `eslint` puliti.
Resta da confermare a video.

---

<!--
Copia il template qui sotto per ogni nuovo bug.
Ricordati di aggiungere anche una riga nella tabella di Riepilogo.

### BUG-XXX — Titolo breve e descrittivo

- **Area / Schermata:**
- **Severità:**
- **Stato:** Aperto
- **Data rilevamento:** AAAA-MM-GG
- **Versione:** 1.1.1

**Descrizione**
Cosa succede, in sintesi.

**Passi per riprodurre**
1.
2.
3.

**Comportamento atteso**
Cosa ci si aspettava.

**Comportamento osservato**
Cosa è successo realmente.

**Frequenza:** sempre / intermittente / una volta
**Allegati:** screenshot, log, video (path o link)
**Note tecniche:** eventuali messaggi d'errore, console log, stacktrace

---
-->
