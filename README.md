# New Benchmark JPEG-XL Tool
### (WIP)

### Usage
```
$ cd benchmark-jpegxl
$ cargo build
$ ./target/debug/benchmark-jpegxl -h
$ ./target/debug/benchmark-jpegxl         // runs normally, producing new results in a location like `./benchmarks/7`
$ ./target/debug/benchmark-jpegxl -c -t   // cleans all existing temp results (`rm -rf ./benchmarks/temp/*`) and then generates new temp results in a location like `./benchmarks/temp/7`
```

## Old Test JXL Script
### (deprecated)

### Usage Examples
```
$ python3 old_test_jxl.py --help    # show man page
$ python3 old_test_jxl.py -t=/path/to/test-set/ -a -g    # run compression, decompression, and comparision stages on test-set/'s images, graph anything applicable
$ python3 old_test_jxl.py --test_set=/path/to/test-set/ -all --graph    # run compression, decompression, and comparision stages on test-set/'s images, graph anything applicable
$ python3 old_test_jxl.py --test_set=/path/to/test-set/ --compress    # run just the compression stage on test-set/'s images, do not graph anything
```
