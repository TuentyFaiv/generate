# tfverse

> **Status: v2 rewrite, usable locally.** Nothing is published from this branch yet; install it from source to try it.
> v1 (`@tuentyfaiv/cli` ≤ 0.0.11) is being sunset; see the migration notes once v2 ships.

tfverse generates projects and code from **packs** — for humans at the terminal and for AI agents over MCP. A pack is
a versioned bundle of starters, pieces and presets that anyone can publish from a git tag, a tarball URL or a local
folder. Code a pack adds is yours to edit, and `tfverse update` still merges upstream fixes into it. Pieces adapt to the
project's stack, declare typed inputs (prompts, flags and MCP schema from one definition) and can patch existing files.
Packs are verified by hash and optional signature, installs are dependency-free, and every command works
non-interactively.

## Try it

```sh
cargo install --path crates/tfverse --locked   # puts `tfverse` in ~/.cargo/bin

cd my-react-or-svelte-app
tfverse pack list                       # official packs: ui (atoms, molecules, organisms), backend
tfverse add ui/atom Button --dry-run    # see the plan; nothing is written
tfverse add ui/atom Button              # asks before applying; --yes skips the question
tfverse status                          # clean / modified / missing per generated file
tfverse why src/ui/atoms/button/button.tsx:3
tfverse undo                            # reverts the last apply, refusing to drop later edits
```

More commands:

| Command                                                           | What it does                                                       |
| ----------------------------------------------------------------- | ------------------------------------------------------------------ |
| `tfverse add <pack/piece> [name] -i key=value --from <source>`    | Generate a piece, loading its pack from a source first             |
| `tfverse adopt <pack/piece> [name]`                               | Take over existing code so updates merge into it                   |
| `tfverse update <pack> [--from <source>] [--on-conflict markers]` | Move to a new pack version, merging your edits                     |
| `tfverse pack add <source>`                                       | Load and verify `gh:`, `git:`, `https://`, `registry:`, `./` packs |
| `tfverse pack capture <files…> --piece <name> --name UserCard`    | Turn project code into a piece of `packs/local`                    |
| `tfverse log`, `tfverse detect`, `tfverse pack show <pack>`       | History, detected stack, a pack's pieces and inputs                |

Every command takes `--json` and `--no-input` for scripts and agents; `--dry-run`, `--diff` and `--review`
(keep or skip each hunk, like `git add -p`) work wherever a plan is applied. Exit codes: `0` done, `1` failed,
`2` usage or confirmation needed, `3` plan rejected.

For AI agents, `tfverse mcp` serves the same plan → review → apply flow over MCP, e.g.
`claude mcp add tfverse -- tfverse mcp`. Nothing is written until the agent calls `apply_plan`.

## Repository layout

```text
crates/
  tfverse-core/      engine, in layers: model → template → patch → pack → workspace
                     (files, config, lockfile) → history → plan → engine
  tfverse-packs/     pack sources, in layers: spec → transport (HTTPS, git over SSH, credentials)
                     → format (archives, registry items) → signing → store (cache, official
                     packs, library) → resolver
  tfverse-rpc/       stdio JSON-RPC API over the engine (GUI, tools)
  tfverse-mcp/       MCP server over stdio, one tool per engine call
  tfverse/           the CLI: cli (arguments) and ui (output, prompts, review) → inputs and
                     context → commands
  xtask/             repository automation (`cargo xtask <task>`)
packs/
  ui/                official atomic UI pieces: atom, molecule, organism (web, Rust, native)
  backend/           official backend pieces: resource, error, middleware, database
schemas/             JSON Schemas for pack.toml, piece.toml, tfverse.lock and config
apps/docs/           legacy v1 docs starter, replaced in Phase 7
```

## Development

Requirements: [rustup](https://rustup.rs) (the pinned toolchain in `rust-toolchain.toml` installs automatically) and
[Bun](https://bun.sh) ≥ 1.4.4 for Biome.

```bash
bun install            # Biome only, for JSON/JS formatting and linting
cargo xtask ci         # everything CI runs: fmt, clippy, rustdoc, tests, cargo-deny, Biome
cargo xtask fmt        # or a single step: fmt | lint | doc | test | deny | js
cargo xtask schema     # regenerate schemas/ after changing a file format
cargo run -- --help    # run the CLI
```

`cargo xtask deny` needs [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) on your `PATH`.

### Conventions

- Formatting matches `biome.json`: hard tabs, width 2, LF, 120 columns (`rustfmt.toml` mirrors it).
- Clippy runs with warnings denied; `unwrap`, `expect`, `panic`, `todo` and `dbg!` are errors outside tests.
- Libraries use `thiserror`; only the binary uses `anyhow`.
- Every product name comes from `tfverse_core::brand`, so the planned rename stays a one-line change.

## Writing a pack

A pack is a folder with a `pack.toml` and one folder per piece:

```text
my-pack/
├─ pack.toml                 # name, version, permissions
└─ pieces/component/
   ├─ piece.toml             # inputs, variants, file rules, patches
   └─ files/react/my-example.tsx
```

Templates are **real code**. Each input names an `example` that appears in the templates; tfverse
replaces it in every case form (`MyExample`, `myExample`, `my-example`, `my_example`, `MY_EXAMPLE`,
`my/example`, …) and as written (`com.example.app`), file names included. Sentence case
(`My example`) is not a case form, so write examples in one of those forms. Optional code uses comment
directives in any comment style:

```tsx
import styles from "./my-example.module.css"; // @tfverse if styled

// @tfverse if styled
return <h2 className={styles.myExample} />;
// @tfverse else
return <h2 className="my-example" />;
// @tfverse end
```

A variant can set its own `target` (Flutter writes to `lib/`, Compose to `app/src/main/java/…`); an
empty `target` writes from the project root, for pieces that also add files under `test/` or `tests/`.

Templates cannot read files or the environment, or run code. A pack can only write the paths its
`permissions.write` globs declare, never `.git/` or tfverse's own state, and every plan flags risky
paths such as `package.json`, CI workflows or agent configuration.

A piece can build on other pieces of its pack and name the packages its code imports:

```toml
requires = ["error"]          # added first when the project does not have it yet

[[dependencies]]
ecosystem = "npm"             # npm, cargo, pypi, go, pub, rubygems or packagist
name = "zod"
version = "^4.1"
variants = ["hono", "express"]
```

Plans list dependencies; tfverse never installs them on its own. Declaring them needs
`permissions.dependencies = true` in `pack.toml`, and the user can refuse them.

Patches edit files the project already has: `anchor` inserts text next to a unique line, `import` adds an import,
and `css_vars` adds custom properties to a CSS block, creating the block if needed and keeping values already set:

```toml
[[patches]]
kind = "css_vars"
file = "src/app/globals.css"
selector = ":root"            # or `.dark`, `@theme inline`, …
vars = { sidebar-border = "220 13% 91%" }
```

### Pack sources

| Source                                      | Example                                                                             |
| ------------------------------------------- | ----------------------------------------------------------------------------------- |
| Official (built into the binary)            | `ui`, `official:backend`                                                            |
| Folder                                      | `./packs/mine`, `path:/opt/packs/mine`                                              |
| GitHub tag or commit                        | `gh:acme/packs@v1.2.0`, `gh:acme/packs/ui@v1.2.0`                                   |
| HTTPS tarball                               | `https://example.com/ui-1.2.0.tar.gz`                                               |
| Registry item (`registry-item.json` format) | `registry:https://acme.dev/r/card.json`, `registry:@acme/card`                      |
| Any git repository over SSH                 | `git:git@github.com:acme/packs.git@v1.2.0`, `git:ssh://git@host/x.git//packs/ui@v2` |
| Namespace from the project config           | `@acme/ui`, with `"sources": { "@acme": "gh:acme/packs@v1.2.0" }`                   |

Downloads are HTTPS only and verified against the operating system's trusted certificates. The lockfile
records each pack's source and the SHA-256 of its files: a pinned pack loads from the cache offline, and a
source that now serves different files for the same pin is refused. Network sources come from the `remote` feature of
`tfverse-packs`, on by default; `--no-default-features` leaves out the HTTPS crates and refuses `git:`, and remote
sources then load only from the cache. Official packs ship with tfverse, so a newer tfverse brings their next version.

Private sources use credentials from your user config (`config.toml` in the tfverse config folder), never from the
project config, so a cloned project cannot send your tokens elsewhere. Only the name of an environment variable is
stored:

```toml
[auth."github.com"]
env = "GITHUB_TOKEN" # sent as `Authorization: Bearer …`; `header` and `scheme` change that
```

With a token, `gh:` sources download through the GitHub API. Tokens go only to their own host, never to a host a
download redirects to. `git:` sources work over SSH only and run your own `git`, so your SSH keys, agent and
`~/.ssh/config` apply; this is how GitLab, Gitea and self-hosted repositories are reached. tfverse never prompts
(SSH runs in batch mode unless you set your own SSH command), stops git after two minutes, and uses `git archive`,
so nothing is checked out.

### Registry items

`registry:<url>` turns an item in the `registry-item.json` format into a pack named `registry-<item>`: the item and
every item it requires become pieces, and its npm packages are listed in the plan. Required items named without a
URL are fetched from beside the item that requires them. Where files go and how imports read comes from the
project config:

```json
{
	"registry": {
		"aliases": { "components": "@/components", "utils": "@/lib/utils" },
		"css": "src/app/globals.css",
		"server_components": true,
		"namespaces": { "@acme": "https://acme.dev/r/{name}.json" }
	}
}
```

The pack's version comes from the fetched content, so `tfverse update registry-<item> --from <same source>` fetches
the item again and merges it like any other pack. CSS variables go into `registry.css` (`light` in `:root`, `dark`
in `.dark`, `theme` in `@theme inline`), keeping values the project already set; `css`, `tailwind` and `envVars`
are not applied yet, and the pack description names them.

### Signatures

A pack can be signed with [minisign](https://jedisct1.github.io/minisign/): sign the one-line message
`tfverse pack <name> <version> <digest>` and put the signature in `pack.minisig` at the pack root (the signature is
not part of the digest). Trust keys in your user config:

```toml
[signing]
require = true                 # refuse unsigned packs and packs signed by keys you have not trusted
keys = { acme = "RWQ…" }       # minisign public keys
revoked = ["1C3B7AE36BE79191"] # key ids refused even when the signature is valid
```

A bad signature or a revoked key is always refused; tfverse also ships its own revocation list. The lockfile records
the key that signed each pack, and a later version signed by another key, or not signed, is refused. Official packs
ship inside tfverse and need no signature.

### Updating generated code

An update moves every piece from a pack to another version of it, as one plan you review before applying:

| The file since generation       | What the update does                                                         |
| ------------------------------- | ---------------------------------------------------------------------------- |
| Unedited                        | Takes the new version, or removes it if the new version dropped it           |
| Edited on other lines           | Merges both, line by line                                                    |
| Edited on the same lines        | Conflict: refuse (default), write Git-style markers, keep yours, take theirs |
| Edited, and dropped by the pack | Stays as you left it and is no longer tracked                                |
| Deleted by you                  | Stays deleted                                                                |

New files and newly required pieces are added, patch regions tfverse inserted are replaced unless you edited them,
and the merge base is the original tfverse wrote: cached in `.tfverse/`, or rebuilt from the old pack version.
Updates are transactions like any other, so undo restores the previous version and lockfile.

A pack can ship migrations for code it does not own, run when an update crosses their version and shown in the
plan like any other change:

```toml
[[migrations]]
to = "2.0.0"
description = "Button's `kind` prop is now `variant`"
files = ["src/**/*.tsx"]                # must also be in `permissions.write`
ast_grep = "migrations/2.0.0-button.yml" # or: rename = { from = "OldName", to = "NewName" }
```

`rename` is built in and matches whole words. `ast_grep` rules run with [ast-grep](https://ast-grep.github.io)
(`ast-grep` on `PATH`, or `ast_grep = { command = [...] }` in your user config); it reads each file on stdin and
tfverse applies the edits it reports, so ast-grep never writes to the project. Files the update itself changes are
skipped and noted, and an update can run without migrations.

Any plan can be reviewed hunk by hunk before applying, like `git add -p`: keep a file's whole change, none of it,
or only some hunks. A skipped hunk counts as your edit, so later updates keep it skipped; a skipped new file is left
out of the lockfile, so the next update offers it again.

## Official packs

`packs/` holds the default packs; `tests/official_packs.rs` renders every piece in every variant.

- **`ui`**: atomic design pieces in `src/ui/{atoms,molecules,organisms}/<name>/`, with typed props, accessible
  markup, CSS (or Tailwind when detected) and tests, for React, Solid, Qwik, Svelte, Vue, Astro, Angular, Lit,
  Leptos, Dioxus, Yew, Flutter, Jetpack Compose (Android and Multiplatform) and SwiftUI.
- **`backend`**: `resource` (model → repository → service → routes, plus tests), `error` (RFC 9457 problem
  details), `middleware` (request id style headers) and `database`, for Hono, Express, Axum, FastAPI, chi and
  Echo. Repositories follow the detected ORM (Drizzle, Prisma, sqlx, SQLAlchemy, GORM) and always include an
  in-memory one for tests.

File names are kebab-case wherever the language allows it (`user-card.tsx`, `user-card.module.css`). Where the
language or its standard tooling fixes the name, the pack follows it: snake_case for Rust, Python, Go and Dart
modules, and PascalCase for Swift and Kotlin types.

## License

[MIT](LICENSE) © Tonalli López
