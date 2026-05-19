# rarc

rarc is a small wrapper around clang for producing RISC-V assembly compatible with RARS simulator.

## Motivation

RARS is a simple assembler simulator, which does not support linking and only supports a few assembler directives. This project provides:
- A postprocessor, which converts assembler output to be compatible with RARS assembler subset.
- A minimal header-only implementation of libc.

## Installation

See [INSTALL.md](INSTALL.md) for detailed installation instructions.

## Usage

### Compiling files

```bash
rarc input.c -o out.s
```

Show help or version:

```bash
rarc --help
rarc --version
```

You can pass flags that will be provided to clang after `--`:

```bash
rarc src/main.c -o out.s -- -DDEBUG
```

### Writing code

To write code that rarc can understand, you need to make sure that libc subset provided by rarc provides the features you use in your code. See [nolibc readme](INSTALL.md) for more details.

Please note that rarc produces a simple assembly file, that RARS links itself. That limits your ablilties for writing a C program. Especially, note that if you use nolibc in two C files, then compile them using rarc and run together, RARS may complain about ambigious labels (each file produced by rarc carries all the nolibc code it needs).

You can use RARS enviroment calls inside your program. Here is an example:
```c
#include <rars.h>

// later
syscall(RARS_PrintString, "Hello World!");
```

## Configuration

rarc reads configuration from the first existing file in these locations:

- `$XDG_CONFIG_HOME/rarc/config.toml`
- `$HOME/.config/rarc/config.toml`
- `$XDG_CONFIG_DIRS/rarc/config.toml` (defaults to `/etc/xdg`)

Supported keys (all optional):

- `compiler_path` (default: `clang`)
- `clang_args` (default RISC-V flags)
- `supported_directives`
- `default_out_name` (default: `out.s`)
- `include_paths` (extra `-I` include dirs)

Include paths are appended to clang arguments as `-I<path>`.

## Examples

You can browse some examples in the `integration-tests/fixtures/cases` folder.

## Tests

```bash
cargo test -p rarc
cargo test -p integration-tests
```

Integration tests can be configured with environment variables like:

```bash
RARC_TEST_RARS_JAR=/path/to/rars.jar cargo test -p integration-tests
```

See [integration-tests/README.md](integration-tests/README.md) for details.

## Licensing

This project is distributed under MIT Lisence.