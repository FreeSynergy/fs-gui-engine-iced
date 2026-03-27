# fs-gui-engine-iced — Claude Session Rules

## Purpose
iced render engine implementing fs-render traits.
Application code must NEVER import this crate directly — only fs-render.

## Quality Gates (every commit)
1. Design Pattern → Structs/Traits → cargo check → Impl → clippy → fmt → test → commit
2. `#![deny(clippy::all, clippy::pedantic, warnings)]` in lib.rs
3. `cargo clippy --all-targets -- -D warnings` → 0 warnings
4. `cargo fmt --check` → clean
5. `cargo test` → all green
6. `cargo build --release` → works

## OOP Rules
- Traits statt match-Blöcke
- Objekte statt Daten
- IcedWindow, IcedWidget, IcedTheme sind Objekte mit Verhalten — keine data bags

## i18n
- Kein roher String im Code — alles über fs-i18n FTL-Keys

## After changes
- Doku-Seite in fs-documentation/de/ aktualisieren
- commit + push beide Repos (fs-gui-engine-iced + fs-documentation)

## No Co-Authored-By Anthropic in commits
