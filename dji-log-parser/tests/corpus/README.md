# Flight log corpus

A wide set of real DJI flight logs found on GitHub, used to measure how well the parser handles
each log version and aircraft. Unlike [`../fixtures`](../fixtures/README.md), these logs are not
part of the repository: most of them have no license allowing redistribution. They are downloaded
from their source and checked against their sha256.

`sources.json` lists each log with its url (pinned to a commit), version, aircraft, source and
license (`null` when the source has no license).

## Running

```sh
DJI_API_KEY=... cargo test --release --test corpus -- --ignored --nocapture
```

- Logs, keychains and the report are cached in `target/dji-log-corpus` (override with
  `DJI_LOG_CORPUS_DIR`). The first run downloads about 70 MB.
- `DJI_API_KEY` is only needed to fetch the keychains of version 13+ logs, once: without it, these
  logs are skipped.

The test prints a summary per version (unknown and invalid records, frames) and writes the full
report, per log, to `target/dji-log-corpus/report.json`. It only fails on panics and parse errors:
unknown records are measured, not asserted.

## Adding a log

Add an entry to `sources.json` with the sha256 of the log (`shasum -a 256`). For a log inside a zip
archive, `url` is the archive and `archive_entry` the path of the log in it.
