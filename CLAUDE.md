# Phosphor — Project Rules

## Code Style
- **NO emojis** anywhere in code, comments, commit messages, or UI strings
- **NO LLM-revealing notes** — no comments like "AI-generated", "TODO: implement", "as an AI", "here's what this does", excessive docstrings explaining obvious logic, or overly verbose inline documentation
- Write code as a senior developer would — clean, concise, self-documenting
- Comments only where logic is genuinely non-obvious
- Commit messages: short, imperative, professional

## Tech Stack
- Tauri v2 (Rust + React 19 + TypeScript + Tailwind CSS v4)
- XState v5 for frontend state machine
- NO component libraries (no shadcn/ui, no Radix) — all custom components
- Font: IBM Plex Mono only
- License: GPL v3
