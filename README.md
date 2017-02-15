audrey [![Build Status](https://travis-ci.org/RustAudio/audrey.svg?branch=master)](https://travis-ci.org/RustAudio/audrey) [![Crates.io](https://img.shields.io/crates/v/audrey.svg)](https://crates.io/crates/audrey) [![Crates.io](https://img.shields.io/crates/l/audrey.svg)](https://github.com/RustAudio/audrey/blob/master/LICENSE-MIT) [![docs.rs](https://docs.rs/audrey/badge.svg)](https://docs.rs/audrey/)
======

A crate to simplify reading, writing and converting between a range of audio formats.

The crate specifically focuses on pure-rust implementations of audio format decoders
and encoders to ensure ease of use, portability, safety and performance.


Supported Formats
-----------------

| Format | Extensions | Read | Write | Cargo Feature | Dependencies |
| ------ | ---------- | ---- | ----- | ------------- | ------------ |
| FLAC | "flac" | YES | - | flac | [claxon](https://crates.io/crates/claxon) |
| Ogg Vorbis | "ogg", "oga" | YES | - | ogg_vorbis | [lewton](https://crates.io/crates/lewton) |
| WAV | "wav", "wave" | YES | - | wav | [hound](https://crates.io/crates/hound) |
| ALAC (within CAF) | "caf" | YES | - | caf_alac | [caf](https://crates.io/crates/caf) [alac](https://crates.io/crates/alac) |

All supported formats are enabled by default, however you can hand-pick only the
formats you require using cargo features. For example, if you only required the
WAV and FLAC formats, you could use the `--no-default-features` and `--features
"wav flac"` flags when building with cargo.


License
-------

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.


**Contributions**

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
