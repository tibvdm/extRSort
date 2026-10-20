# extRSort

An external merge sort for large text files. It reads lines from stdin and writes them to stdout in
byte order, the same order as `LC_ALL=C sort`. Input that does not fit in memory is sorted in chunks
on several threads, written to temporary files, and merged.

## Build

```sh
cargo build --release
```

The binary is `target/release/sorter`.

## Usage

```sh
sorter [OPTIONS] < input.tsv > sorted.tsv
```

| Option | Default | Meaning |
|---|---|---|
| `-b`, `--buffer-size` | `400MB` | Memory for the lines: bytes, or a size such as `400MB`, `2G` (2 × 10⁹) or `2GiB` (2 × 2³⁰) |
| `-p`, `--parallel` | `4` | Number of threads |
| `-t`, `--temp-dir` | `/tmp/sort-rs` | Where the temporary files go. It is made when it does not exist |
| `-f`, `--field` | `1` | Sort from this field to the end of the line |
| `-d`, `--delimiter` | tab | The field delimiter |

Notes:

- `-f N` compares the line from the delimiter before field N to the end of the line, as
  `sort -k N` does, not `sort -k N,N`. Lines with equal keys come out in any order.
- GNU sort reads `-S 2G` as 2 × 2³⁰ bytes; here `2G` is 2 × 10⁹ bytes and `2GiB` is 2 × 2³⁰.
- The memory use is about `-b` plus a fixed overhead of roughly 100 MB.
- Every output line ends in a newline, also when the last input line did not.
- On an error, such as a full disk, the sorter prints it, removes its temporary files and exits
  with a non-zero status. SIGINT and SIGTERM also remove the temporary files. SIGHUP is not
  handled, so a sorter started with `nohup` keeps running after a logout.

## Tests

```sh
cargo test
```

`tests/sort.rs` compares the output with the same lines sorted in Rust. `bench.sh` compares the
speed with GNU sort.
