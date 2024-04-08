# dji-log-parser

[![crates](https://img.shields.io/crates/v/dji-log-parser.svg)](https://crates.io/crates/dji-log-parser)
[![docs.rs](https://docs.rs/dji-log-parser/badge.svg)](https://docs.rs/dji-log-parser)

A library / cli for parsing DJI txt logs

## Features

- Supports all log versions and encryptions
- Parse records and extract embedded images
- Export flight tracks to GeoJSON and KML

## Encryption in Version 13 and Later

Starting with version 13, log records are AES encrypted and require a specific keychain for decryption. This keychain must be obtained from DJI using their API. An apiKey is necessary to access the DJI API.

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
dji-log DJIFlightRecord_YYYY-MM-DD_\[00-00-00\].txt --api-key __DJI_API_KEY__ > records.json
```

or with an output arg

```bash
dji-log DJIFlightRecord_YYYY-MM-DD_\[00-00-00\].txt --api-key __DJI_API_KEY__ --output records.json
```

### With image / thumbnails extraction

Use `%d` in the images or thumbnails option to specify a sequence.

```bash
dji-log DJIFlightRecord_YYYY-MM-DD_\[00-00-00\].txt --api-key __DJI_API_KEY__ --images image%d.jpeg --thumbnails thumbnail%d.jpeg --output records.json
```

### With kml generation

```bash
dji-log DJIFlightRecord_YYYY-MM-DD_\[00-00-00\].txt --api-key __DJI_API_KEY__ --kml track.kml --output records.json
```

### With geojson generation

```bash
dji-log DJIFlightRecord_YYYY-MM-DD_\[00-00-00\].txt --api-key __DJI_API_KEY__ --geojson track.json --output records.json
```

## Library Usage

### Initialization

Initialize a `DJILog` instance from a byte slice to access version information and metadata:

```rust
let parser = DJILog::from_bytes(&bytes).unwrap();
println!("Version: {:?}", parser.version);
println!("Info: {:?}", parser.info);
```

### Accessing Records

Decrypt records based on the log file version.
For versions prior to 13:

```rust
let records = parser.records(DecryptMethod::None);
```

For version 13 and later:

```rust
let records = parser.records(DecryptMethod::ApiKey("__DJI_API_KEY__"));
```

### Advanced: Manual Keychain Retrieval

For WASM or custom server communication, the library exposes the internal keychain retrieval process:

```rust
// 1. Build a keychain request
let keychain_request = parser.keychain_request().unwrap();

// 2. Fetch from DJI's servers, take a callback as second argument
let keychains = keychain_request.fetch("__DJI_API_KEY__", |result| {
    let keychains = result.unwrap();
});

// 3. Parse records
let records = parser.records(DecryptMethod::Keychains(keychains));

```

Note: Replace `__DJI_API_KEY__` with your actual apiKey

For more information, including a more detailed overview of the log format, [visit the documentation](https://docs.rs/dji-log-parser).

## License

dji-log-parser is available under the MIT license. See the LICENSE.txt file for more info.
