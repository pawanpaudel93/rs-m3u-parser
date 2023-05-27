# M3U Parser

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/m3u_parser.svg)](https://crates.io/crates/m3u_parser)
[![Documentation](https://docs.rs/m3u_parser/badge.svg)](https://docs.rs/m3u_parser)

A library for parsing and manipulating M3U playlists.

## Features

- Parse M3U playlists from files or URLs.
- Extract stream information such as title, logo, URL, category, etc.
- Filter stream information based on attributes and conditions.
- Check the availability of streams by sending requests to their URLs.
- Save filtered stream information to a new M3U playlist.

## Installation

Add the `m3u_parser` crate to your `Cargo.toml` file:

```toml
[dependencies]
m3u_parser = "0.1.2"
```

Import the M3uParser struct and use it to parse M3U playlists:

```rust
use m3u_parser::M3uParser;

#[tokio::main]
async fn main() {
    let mut parser = M3uParser::new(None);
    parser.parse_m3u("path/to/playlist.m3u", false, true).await;
    // Perform operations on the parsed stream information
}
```

For more examples and detailed documentation, see the [API documentation](https://docs.rs/m3u_parser).

## Examples

Parse an M3U playlist file and print the stream information:

```rust
use m3u_parser::M3uParser;

#[tokio::main]
async fn main() {
    let mut parser = M3uParser::new(None);
    parser.parse_m3u("path/to/playlist.m3u", false, true).await;
    for stream_info in parser.streams_info {
        println!("{:?}", stream_info);
    }
}
```

## Other Implementations

- `Golang`: [go-m3u-parser](https://github.com/pawanpaudel93/go-m3u-parser)
- `Python`: [m3u-parser](https://github.com/pawanpaudel93/m3u-parser)
- `Typescript`: [ts-m3u-parser](https://github.com/pawanpaudel93/ts-m3u-parser)

## Author

👤 **Pawan Paudel**

- Github: [@pawanpaudel93](https://github.com/pawanpaudel93)

## 🤝 Contributing

Contributions, issues and feature requests are welcome!<br />Feel free to check [issues page](https://github.com/pawanpaudel93/rs-m3u-parser/issues).

## Show your support

Give a ⭐️ if this project helped you!

Copyright © 2023 [Pawan Paudel](https://github.com/pawanpaudel93).
