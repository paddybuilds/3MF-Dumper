# 3MF Dumper

Rust CLI for decompiling `.3mf` archives into readable folder structures.

### Decompile one or more 3MF files

```powershell
cargo run -- decompile .\example.3mf
```

```powershell
cargo run -- decompile .\a.3mf .\b.3mf --out-dir .\out --jobs 8 --pretty-xml
```

Options:

- `--out-dir`: output directory (default: `decompiled`)
- `--overwrite`: replace existing output directory
- `--pretty-xml`: pretty-format XML files while extracting
- `--jobs N`: number of parallel worker threads
