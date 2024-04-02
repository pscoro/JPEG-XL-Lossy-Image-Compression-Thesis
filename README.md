# JPEG XL Lossy Image Compression
## Thesis

## Benchmarking JPEG XL
### Build and Run `benchmark-jpegxl` source
Prerequisites:
- have Rust and cargo installed
- have Docker installed and running

#### Step 1 - Set up image testsets
```
cd benchmark-jpegxl
mkdir test_images
```
inside here you can have multiple dataset subfolders if you want. Add your input images here.

#### Step 2 - Build `benchmark-jpegxl` with Rust `cargo`
```
cd benchmark-jpegxl
cargo build
```

#### Step 3 - Run the benchmarker
```
./target/debug/benchmark-jpegxl --help
```
