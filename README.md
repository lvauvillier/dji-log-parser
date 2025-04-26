# dji-log-parser

[![crates](https://img.shields.io/crates/v/dji-log-parser.svg)](https://crates.io/crates/dji-log-parser)
[![docs.rs](https://docs.rs/dji-log-parser/badge.svg)](https://docs.rs/dji-log-parser)

A library and CLI tool for parsing DJI txt logs with support for all log versions and encryptions.

## Features

- Parse records and extract embedded images from DJI logs
- Normalize records across different log versions for a consistent frame format
- Export frames to CSV for easy analysis
- Generate flight tracks in GeoJSON and KML formats
- Support for all log versions, including encrypted logs (version 13+)

## Encryption in Version 13 and Later

Starting with version 13, log records are AES encrypted and require a specific keychain for decryption. This keychain must be obtained from DJI using their API. An apiKey is necessary to access the DJI API.

Once keychains are retrieved from DJI API, they can be stored along with the original log for further offline use.

### Obtaining an ApiKey

To acquire an apiKey, follow these steps:

1. Visit [DJI Developer Technologies](https://developer.dji.com/user) and log in.
2. Click `CREATE APP`, choose `Open API` as the App Type, and provide the necessary details like `App Name`, `Category`, and `Description`.
3. After creating the app, activate it through the link sent to your email.
4. On your developer user page, find your app's details to retrieve the ApiKey (labeled as the SDK key).

## Cli Usage

### Installation

[Download](https://github.com/lvauvillier/dji-log-parser/releases) binary from latest release

### Basic usage

```bash
dji-log --api-key __DJI_API_KEY__ DJIFlightRecord.txt > frames.json
```

or with an output arg

```bash
dji-log --api-key __DJI_API_KEY__ --output frames.json DJIFlightRecord.txt
```

### Additional Options

- `--raw`: Export raw records instead of normalized frames
- `--images image%d.jpeg`: Extract embedded images
- `--thumbnails thumbnail%d.jpeg`: Extract thumbnails
- `--csv`: Generate a CSV file of frames
- `--kml track.kml`: Generate a KML file of the flight track
- `--geojson track.json`: Generate a GeoJSON file of the flight track

Use `%d` in the images or thumbnails option to specify a sequence.

### Advanced Options

- `--api-custom-department`: Manually set the department on keychains apis request
- `--api-custom-version`: Manually set the department on keychains apis request

For a complete list of options, run:

```bash
dji-log --help
```

## Library Usage

### Initialization

Initialize a `DJILog` instance from a byte slice to access version information and metadata:

```rust
let parser = DJILog::from_bytes(bytes).unwrap();
```

### Access general data

General data are not encrypted and can be accessed from the parser for all log versions:

```rust
// Print the log version
println!("Version: {:?}", parser.version);

// Print the log details section
println!("Details: {}", parser.details);
```

### Retrieve keychains

For logs version 13 and later, keychains must be retrieved from the DJI API to decode the records:

```js
// Replace `__DJI_API_KEY__` with your actual apiKey
let keychains = parser.fetch_keychains("__DJI_API_KEY__").unwrap();
```

Keychains can be retrieved once, serialized, and stored along with the log file for future offline use.

### Accessing Frames

Decrypt frames based on the log file version.

A `Frame` is a standardized representation of log data, normalized across different log versions.
It provides a consistent and easy-to-use format for analyzing and processing DJI log information.

For versions prior to 13:

```rust
let frames = parser.frames(None);
```

For version 13 and later:

```rust
let frames = parser.frames(Some(keychains));
```

### Accessing raw Records

Decrypt raw records based on the log file version.
For versions prior to 13:

```rust
let records = parser.records(None);
```

For version 13 and later:

```rust
let records = parser.records(Some(keychains));
```

For more information, including a more detailed overview of the log format, [visit the documentation](https://docs.rs/dji-log-parser).

## License

dji-log-parser is available under the MIT license. See the LICENSE.txt file for more info.
