# Marketing Site

This directory is for the public website only.

## Scope
- Landing page, docs links, legal pages, pricing copy
- No product state, no chat runtime, no self-host control plane

## Rules
- Keep this deployable as static assets.
- Keep product app code out of this tree.

## Active Site
- `site/` contains the current static marketing pages.

## Local Run

```bash
cd apps/marketing/site
python3 -m http.server 8080
```
