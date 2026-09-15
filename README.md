# zono

A tiny **cross-platform todo list** written in Rust. No crates, no runtime, no config:
just `std`, a single binary, and a plain TSV file in `~/.zono/tasks.tsv`.

## Features

- add, toggle, remove tasks, and clear everything you finished
- state persists between runs in a human-readable TSV file
- runs identically on Linux, macOS and Windows
- zero dependencies, so it builds anywhere a stock Rust toolchain exists

## Commands

| command | what it does |
| --- | --- |
| `a <title>` | add a task |
| `d <id>` | toggle done / not done |
| `r <id>` | remove a task |
| `c` | clear every finished task |
| `h` | help |
| `q` | save and quit |

## Build and run

```bash
cargo run --release
```

Prebuilt binaries land in [Releases](https://github.com/supercubegame/zono/releases) once the
workflow in `ci/release.yml` is moved to `.github/workflows/release.yml`.

## The interface

```
+----------------------------------------------+
|  zono  ::  a tiny cross-platform todo list    |
|  v0.1.0 :: rust, zero dependencies            |
+----------------------------------------------+

  YOUR TASKS
  ----------------------------------------------
    1. [x] Ship zono v0.1.0
    3. [ ] Wire up the release pipeline
  ----------------------------------------------
  1 of 2 done

zono>
```

## Storage

`~/.zono/tasks.tsv` (override the base directory with `ZONO_HOME`).

## License

MIT
