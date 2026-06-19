# Installation

Gitronics is distributed as a Python package containing a compiled Rust extension. Pre-built wheels are published to [PyPI](https://pypi.org/project/gitronics/) for the most common platforms; no Rust toolchain is required for end users.

## Requirements

- Python **3.9** or later
- A supported platform: Linux x86\_64 / aarch64, macOS x86\_64 / Apple Silicon, Windows x86\_64

## Install from PyPI

```bash
pip install gitronics
```

After installation the `gitronics` command is available in your shell:

```bash
gitronics --help
```

!!! note
    Some corporate computers do not allow the execution of non-whitelisted executable files. If when running `gitronics` you get an error like `Access denied` you could try running the command as a Python module instead:
    ```bash
    python -m gitronics --help
    ```

## Install from source

Building from source requires:

- [Rust](https://rustup.rs/) (stable toolchain)
- [maturin](https://github.com/PyO3/maturin) ≥ 1.0

```bash
git clone https://github.com/Fusion4Energy/gitronics
cd gitronics
pip install maturin
maturin develop --release
```

## Building the documentation locally

```bash
pip install -r docs/requirements.txt
mkdocs serve
```

Open [http://127.0.0.1:8000](http://127.0.0.1:8000) in your browser.
