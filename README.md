# fs-gui-engine-iced

iced / libcosmic render engine for FreeSynergy — implements all
[fs-render](../fs-render) abstraction traits.

## Purpose

This crate provides the standard GUI backend for the FreeSynergy Desktop.
It bridges the renderer-agnostic `fs-render` API to the concrete `iced 0.13`
runtime.  Application code in `fs-desktop` and `fs-apps` only imports
`fs-render` — never this crate.

## Architecture

```
fs-render (traits)
    └── fs-gui-engine-iced (this crate)
            IcedEngine  → implements RenderEngine
            IcedWindow  → implements FsWindow
            IcedWidget  → implements FsWidget
            IcedTheme   → implements FsTheme
```

### libcosmic integration (planned)

The current build uses vanilla `iced 0.13`.  A full
[libcosmic](https://github.com/pop-os/libcosmic) integration (Pop!_OS COSMIC
design system, system palette, portal support) is planned for the G2.8 phase
when `fs-desktop` adopts this engine via feature flag.

## Build

```sh
cargo build --release
cargo test
```

## License

MIT — see [LICENSE](LICENSE).
