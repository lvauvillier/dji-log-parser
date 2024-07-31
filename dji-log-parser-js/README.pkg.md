# dji-log-parser-js

A powerful JavaScript library for parsing DJI txt logs with support for all log versions and encryptions.

## Features

- Parse records and extract embedded images from DJI logs
- Normalize records across different log versions for a consistent frame format
- Support for all log versions, including encrypted logs (version 13+)
- Easy-to-use API for accessing general data, frames, and raw records
- Ability to fetch and store keychains for offline use with encrypted logs
- Browser support ([See Limitations](#limitations))

## Installation

To install the DJI Log Parser library, use npm:

```bash
npm install dji-log-parser-js
```

Or using yarn:

```bash
yarn add dji-log-parser-js
```

## Encryption in Version 13 and Later

Starting with version 13, log records are AES encrypted and require a specific keychain for decryption. This keychain must be obtained from DJI using their API. An apiKey is necessary to access the DJI API.

Once keychains are retrieved from DJI API, they can be stored along with the original log for further offline use.

### Obtaining an ApiKey

To acquire an apiKey, follow these steps:

1. Visit [DJI Developer Technologies](https://developer.dji.com/user) and log in.
2. Click `CREATE APP`, choose `Open API` as the App Type, and provide the necessary details like `App Name`, `Category`, and `Description`.
3. After creating the app, activate it through the link sent to your email.
4. On your developer user page, find your app's details to retrieve the ApiKey (labeled as the SDK key).

## Library Usage

### Initialization

Initialize a `DJILog` instance from a byte buffer to access version information and metadata:

#### Node

```js
import { DJILog } from "dji-log-parser-js";
import { readFileSync } from "fs";

const buffer = readFileSync("./DJIFlightRecord.txt");
const parser = new DJILog(buffer);
```

#### Browser

Html

```html
<input type="file" id="file" name="file" />
```

Javascript

```js
import { DJILog } from "dji-log-parser-js";

document
  .getElementById("file")
  .addEventListener("change", handleFileSelect, false);

function handleFileSelect(event) {
  const file = event.target.files[0];
  if (file) {
    const reader = new FileReader();
    reader.onload = async function (e) {
      const bytes = new Uint8Array(e.target.result);
      const parser = new DJILog(bytes);
    };
    reader.readAsArrayBuffer(file);
  }
}
```

### Access general data

General data are not encrypted and can be accessed from the parser for all log versions:

```js
// Print the log version
console.log("Version:", parser.version);

// Print the log details section
console.log("Details:", parser.details);
```

### Retrieve keychains

For logs version 13 and later, keychains must be retrieved from the DJI API to decode the records:

```js
// Replace `__DJI_API_KEY__` with your actual apiKey
const keychains = await parser.fetchKeychains("__DJI_API_KEY__");
```

Keychains can be retrieved once, serialized, and stored along with the log file for future offline use.

### Accessing Frames

Decrypt frames based on the log file version.

A `Frame` is a standardized representation of log data, normalized across different log versions.
It provides a consistent and easy-to-use format for analyzing and processing DJI log information.

For versions prior to 13:

```js
const frames = parser.frames();
```

For version 13 and later:

```js
const frames = parser.frames(keychains);
```

### Accessing raw Records

Decrypt raw records based on the log file version.
For versions prior to 13:

```js
const records = parser.records();
```

For version 13 and later:

```js
const records = parser.records(keychains);
```

## Limitations

### NodeJS

This library requires `fetch` to be globally available (Node >= 18.0) for retrieving keychains from the DJI endpoint.

For older Node versions, you can manually retrieve the keychains using an alternative method, such as Axios. The library conveniently provides the necessary data for making keychains request through the `parser.keychainsRequest()` function.

Here’s an example with Axios:

```js
const axios = require("axios");

const body = JSON.stringify(parser.keychainsRequest());

const response = await axios.post(
  "https://dev.dji.com/openapi/v1/flight-records/keychains",
  body,
  {
    headers: {
      "Content-Type": "application/json",
      "Api-Key": "your_api_key_here",
    },
  }
);

const keychains = response.data.data;
```

### Browser

The DJI Keychain API endpoint does not support CORS (Cross-Origin Resource Sharing). This limitation is acknowledged in an open issue on GitHub: https://github.com/dji-sdk/FlightRecordParsingLib/issues/31

While awaiting a resolution, a temporary workaround involves using a CORS proxy server. Below is an example utilizing a free proxy server: https://github.com/Zibri/cloudflare-cors-anywhere

```js
// Replace `__DJI_API_KEY__` with your actual apiKey
// Don't use this in production, only for testing purpose
const keychains = await parser.fetchKeychains(
  "__DJI_API_KEY__",
  "https://test.cors.workers.dev/?https://dev.dji.com/openapi/v1/flight-records/keychains"
);
```

## License

dji-log-parser-js is available under the MIT license. See the LICENSE.txt file for more info.
