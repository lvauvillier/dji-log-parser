# dji-log-parser

Edan Edit: 
Original code found here:
https://github.com/lvauvillier/dji-log-parser

I extended this work and I provide an explanation of the automation process of creating a C-compatible interface of the RUST library for use by other languages.
(nb: original implementation was manual creation of the header file. Now the process is automated as is explained below)

The build.rs file works in conjunction with the c_api.rs file to create the dji-log-parser.h file. 
    . build.rs: is a special RUST file that runs before the main compilation. It's purpose is to generate the C header file automatically.
                : it determines the crate directory and package name.
                : it sets up the output path for the header file. 
                : it uses the cbindgen library to generate C bindings from the RUST code.

The c_api.rs file contains the RUST functions that will be exposed to C.
                : functions marked with #[no_mangle] and pub extern "C" are made available to C.
                : these function use C-compatible types (like c_char instead of RUST's str).


How it works:
    . when the code is run with "cargo build", the build.rs script is executed first.
    . it looks for functions marked with #[no_mangle] and pub extern "C".
    . for each of these functions, it generates a corresponding C function declaration in the header file.

Resulting dji-log-parser.h file.
                ; this file is the interface that other languages use to interact with the RUST library.
                : it declares the functions that are implemented in RUST but callable from C.


The build.rs automates the process of creating a C-compatible interface for the RUST library. It ensures that the header
file always matches the actual implementation in the RUST code, in order to reduce the chance of compile error from manual 
updates.

To use the RUST library ("libdji_log_parser.a") from other languages, include the generated dji-log-parser.h and link against library. 
Golang example:

    package mypackage

    /*
    #cgo LDFLAGS: -L${SRCDIR}/../target/release -ldji_log_parser
    #cgo CFLAGS: -I${SRCDIR}/../dji-log-parser/include
    #include "dji-log-parser.h"
    #include <stdlib.h>
    */
    
    import "C"
    
    <further imports here>
    type GeoJSON struct {
    Type     string    `json:"type"`
    Features []Feature `json:"features"`
    }
    
    type Feature struct {
        Type       string     `json:"type"`
        Geometry   Geometry   `json:"geometry"`
        Properties Properties `json:"properties"`
    }
    
    type Geometry struct {
        Type        string    `json:"type"`
        Coordinates []float64 `json:"coordinates"`
    }
    
    type Properties struct {
        Time   string  `json:"time"`
        Height float64 `json:"height"`
        Speed  float64 `json:"speed"`
    }
    
    func processReader(reader io.Reader, apiKey string) (*geom.Geometry, error) {
        data, err := io.ReadAll(reader)
        if err != nil {
            return nil, fmt.Errorf("error reading data: %s", err)
        }
    
        cData := C.CBytes(data)
        defer C.free(unsafe.Pointer(cData))
        cLength := C.size_t(len(data))
        cApiKey := C.CString(apiKey)
        defer C.free(unsafe.Pointer(cApiKey))
    
        geojsonPtr := C.get_geojson_string_from_bytes((*C.uchar)(unsafe.Pointer(cData)), cLength, cApiKey)
        if geojsonPtr == nil {
            errPtr := C.get_last_error()
            errStr := C.GoString(errPtr)
            C.c_api_free_string(errPtr)
            return nil, fmt.Errorf("failed to get GeoJSON: %s", errStr)
        }
        defer C.c_api_free_string(geojsonPtr)
    
        geojsonStr := C.GoString(geojsonPtr)
    
        var geojson GeoJSON
        err = json.Unmarshal([]byte(geojsonStr), &geojson)
        if err != nil {
            return nil, fmt.Errorf("error parsing GeoJSON: %s", err)
        }
    }
Finish Edit

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

Initialize a `DJILog` instance from an array of bytes to access version information and metadata:

```rust
let parser = DJILog::from_bytes(bytes).unwrap();
println!("Version: {:?}", parser.version);
println!("Details: {:?}", parser.details);
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

For scenarios like caching, offline use, or custom server communication, the library
exposes the internal keychain retrieval process:

```rust
let keychain_request = parser.keychain_request().unwrap();
let keychains = keychain_request.fetch("__DJI_API_KEY__").unwrap();
let records = parser.records(DecryptMethod::Keychains(keychains));
```

Note: Replace `__DJI_API_KEY__` with your actual apiKey

For more information, including a more detailed overview of the log format, [visit the documentation](https://docs.rs/dji-log-parser).

## License

dji-log-parser is available under the MIT license. See the LICENSE.txt file for more info.


