# Design Mockups

Use this folder for operator-loaded design mockups that support implementation specs.

Mockups are unmanaged files, not lifecycle artifacts. The Project Lead may reference them from specs, but LMBrain does not approve, transition, or edit them as managed Markdown artifacts.

## Package Convention

Prefer one folder per mockup:

```text
design/
  checkout-flow/
    index.html
    README.md
    manifest.json
    assets/
```

- `index.html` is the preview entry point.
- `README.md` is optional design context for humans and agents.
- `manifest.json` is optional metadata:

```json
{
  "title": "Checkout flow",
  "description": "Responsive checkout screens and empty/error states."
}
```

Self-contained single `.html` files are also supported directly under this folder.

## Operator Workflow

1. Ask the Project Lead whether a design pass is needed.
2. If it recommends a design specialist, use the normal agent proposal/profile workflow in `agents/`.
3. Create mockups in the design tool of your choice.
4. Copy the finished self-contained mockup package into this folder.
5. Ask the Project Lead to reference the relevant design path in the implementation spec.

LMBrain never starts a design agent automatically.
