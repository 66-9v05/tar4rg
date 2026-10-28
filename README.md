# tar4rg

Converts files/folders to a single PNG image.  
Data is first packed into a tar archive, then encoded into RGB channels.  
It is a simple tool with no interaction and exits **immediately** if anything goes wrong.

## Build

Requires [Rust](https://rustup.rs/) 1.85 or later.

```bash
cargo build --release
```

The binaries (`encoder` and `decoder`) will be located in `target/release/`.

## Usage

### Encode

```console
$ tree
.
|-- decoder
|-- encoder
`-- file.txt

$ ./encoder "file.txt"
finished

$ tree
.
|-- decoder
|-- encoder
|-- file.png
`-- file.txt
```

---
To encode multiple files at once, use: `encoder <path1> <path2>`.

The encoder will write the bytes in R and G channels by default.  
Use `-rg`, `-rb`, `-gb` to choose a mode. They should be placed before paths.  
For example: `./encoder -gb <path>` will generate a cyan PNG.

The encoder will name the PNG after the source file, and place it in the same directory.  
For example: `./file/text.txt` will lead to `./file/text.png`.

The encoder will exit immediately before creating any PNGs if an input path does not exist or a target path is already occupied.

### Decode

```console
$ tree
.
|-- decoder
|-- encoder
`-- file.png

$ ./decoder "file.png"
finished

$ tree
.
|-- decoder
|-- encoder
|-- file.png
`-- file.txt
```

To decode multiple files at once, use: `./decoder <path1> <path2>`.

The decoder will identify the color mode by the first pixel.  
Do not choose a mode manually by inputting `-rg` or anything else.

The decoder will use the stem and extension of source file which are recorded inside the PNG archive, but still place it in the parent directory of the PNG file.

The decoder **cannot** check the target paths before decoding any of the inputs, because the filename is only known after unpacking the tar archive, and the tar archives are unpacked one by one.  
That means some files/folders may have already been created and written **after an error**. They will cause another error in  subsequent turns if you don't delete them.

## Details

<details>

<summary>doesn't affect usage</summary>

1. There is no shape check in this tool, I think. If you keep the order of pixels from left to right, from top to bottom, maybe you can use any rectangle, even a 1-pixel-high PNG. *Maybe*. I didn't try it.

2. Though it has no shape check, it does have a necessary size check. The total number of pixels is square-rooted as side length. The side length should be a multiple of 8.

3. The first pixel has a special color to indicate a color mode. The three channels are all filled with data to avoid an empty byte. The channel with the highest value will not be used. Don't do what you're thinking about -- the color mode identification uses fixed value matching.

4. You must have realized that the arrangement is spiral. Actually the arrangement of tar bytes follows a clockwise order, starting from the top left. This doesn't really serve any practical purpose; it's just for looks. The spare space is filled with black pixels and a pattern.

5. The pattern has a four-color core and is surrounded by black and white pixels. The pattern is 6x6, 36 pixels. The core is red, green, blue, grey in clockwise order, and the black and white pixels are the CRC32C code.

6. The CRC32C code is different between two PNGs from the same file, because they have different tar headers. Specifically, the `mtime` is incorrect sometimes. It set the mtime of my file to 1971. Maybe because I use MinGW-w64 for Rust, not MSVC, and 1601 did a great job. But it doesn't matter; this information will never be read.

</details>

## Changelog

- **v0.1.0** (2026-10-29):
  - Initial release
  - Configured `Cargo.toml`
  - Added `README.md` and `LICENSE` files

## License

This project uses both the MIT and Apache-2.0 licenses.

### Third-Party Dependencies

- [tar](https://crates.io/crates/tar) (MIT OR Apache-2.0)
- [crc32c](https://crates.io/crates/crc32c) (MIT OR Apache-2.0)
- [lodepng](https://crates.io/crates/lodepng) (Zlib)
