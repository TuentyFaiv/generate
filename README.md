# tfverse

> **Status: v2 rewrite in progress (Phase 0 — workspace scaffold).** Nothing is published from this branch yet.
> v1 (`@tuentyfaiv/cli` ≤ 0.0.11) is being sunset; see the migration notes once v2 ships.

tfverse generates projects and code from **packs** — for humans at the terminal and for AI agents over MCP. A pack is
a versioned bundle of starters, pieces and presets that anyone can publish from a git tag, a tarball URL or a local
folder. Code a pack adds is yours to edit, and `tfverse update` still merges upstream fixes into it. Pieces adapt to the
project's stack, declare typed inputs (prompts, flags and MCP schema from one definition) and can patch existing files.
Packs are verified by hash and optional signature, installs are dependency-free, and every command works
non-interactively.

## Repository layout

```text
crates/
  tfverse-core/      pack model, rendering, plan → apply (brand constants live here today)
  tfverse-packs/     pack sources, integrity + signatures, lockfile    (Phase 2)
  tfverse-mcp/       MCP server over stdio                             (Phase 5)
  tfverse/           the CLI binary
  xtask/             repository automation (`cargo xtask <task>`)
apps/docs/           legacy v1 docs starter, replaced in Phase 6
```

## Development

Requirements: [rustup](https://rustup.rs) (the pinned toolchain in `rust-toolchain.toml` installs automatically) and
[Bun](https://bun.sh) ≥ 1.4.4 for Biome.

```bash
bun install            # Biome only, for JSON/JS formatting and linting
cargo xtask ci         # everything CI runs: fmt, clippy, rustdoc, tests, cargo-deny, Biome
cargo xtask fmt        # or a single step: fmt | lint | doc | test | deny | js
cargo run -- --help    # run the CLI
```

`cargo xtask deny` needs [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) on your `PATH`.

### Conventions

- Formatting matches `biome.json`: hard tabs, width 2, LF, 120 columns (`rustfmt.toml` mirrors it).
- Clippy runs with warnings denied; `unwrap`, `expect`, `panic`, `todo` and `dbg!` are errors outside tests.
- Libraries use `thiserror`; only the binary uses `anyhow`.
- Every product name comes from `tfverse_core::brand`, so the planned rename stays a one-line change.

## License

[MIT](LICENSE) © Tonalli López
