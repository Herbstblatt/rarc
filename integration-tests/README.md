# integration-tests

External-tool integration tests for the `rarc` pipeline.

## Layout

- `fixtures/cases/<case_name>/case.toml`: mode and expectations.
- `fixtures/cases/<case_name>/input.c`: input file for `rarc`.
- `fixtures/cases/<case_name>/expected.s`: optional strict snapshot for `mode="asm"`.
- `fixtures/cases/<case_name>/expected.stdout`: required for `mode="run"`.

## Modes

- `mode = "asm"`: run `rarc` and check asm expectations (`must_contain` / `must_not_contain`).
- `mode = "assemble"`: run `rarc`, then run RARS assemble-only check.
- `mode = "run"`: run `rarc`, then run RARS and compare stdout.

## Config

Config file path can be set via `RARC_TEST_CONFIG`, otherwise `config.toml` in this crate root is used if present.

Environment overrides (highest precedence):

- `RARC_TEST_RARS_JAR`
- `RARC_TEST_RARC_XDG_CONFIG_HOME`
- `RARC_TEST_JAVA_BIN`
- `RARC_TEST_RARC_BIN`

See `config.example.toml` for all fields.

`rarc_xdg_config_home` can be used to pass a rarc config directory in the XDG
form that `rarc` expects (for example containing `rarc/config.toml`). The
harness copies that directory into each per-case workdir and launches `rarc`
with `XDG_CONFIG_HOME` set to the copied path.

For repository-local portability, use a relative path in `config.toml`, for
example `rarc_xdg_config_home = "rarc_config"`. Relative paths are resolved
from `integration-tests/`.

## Commands

From `project/`:

```bash
cargo test -p rarc
cargo test -p integration-tests
RARC_TEST_RARS_JAR=/path/to/rars.jar cargo test -p integration-tests
```
