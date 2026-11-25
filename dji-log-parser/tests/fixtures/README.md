# Test fixtures

Real DJI flight logs used by the snapshot, robustness and record tests. They are kept small and limited to
logs under a permissive license: a wider set of logs, downloaded on demand, is described in
[`../corpus`](../corpus/README.md).

Logs from version 13 come with the keychains fetched from the DJI API (`<name>.keychains.json`),
so tests never hit the network.

| File                   | Version | Aircraft           | Source                                                                                         | License                    |
| ---------------------- | ------- | ------------------ | ---------------------------------------------------------------------------------------------- | -------------------------- |
| `v4.txt`               | 4       | Phantom 3 Pro      | [nattfodd/dji_parser](https://github.com/nattfodd/dji_parser)                                  | MIT                        |
| `v8.txt`               | 8       | Phantom 4          | this project                                                                                   | MIT                        |
| `v8_phantom3.txt`      | 8       | Phantom 3 Standard | [chrisvm/node-djiparsetxt](https://github.com/chrisvm/node-djiparsetxt)                        | MIT                        |
| `v8_mavicpro.txt`      | 8       | Mavic Pro          | [VTO Drone Forensics Program](https://cfreds-archive.nist.gov/drone-images.html) DF019         | MIT, see `LICENSE-VTO.txt` |
| `v8_mavicpro_2017.txt` | 8       | Mavic Pro          | [VTO Drone Forensics Program](https://cfreds-archive.nist.gov/drone-images.html) DF020         | MIT, see `LICENSE-VTO.txt` |
| `v11_mavic2.txt`       | 11      | Mavic 2            | [chrisvm/node-djiparsetxt](https://github.com/chrisvm/node-djiparsetxt)                        | MIT                        |
| `v11_spark.txt`        | 11      | Spark              | [chrisvm/node-djiparsetxt](https://github.com/chrisvm/node-djiparsetxt)                        | MIT                        |
| `v14_mini4pro.txt`     | 14      | Mini 4 Pro         | [rembish/pydjirecord](https://github.com/rembish/pydjirecord)                                  | MIT                        |
