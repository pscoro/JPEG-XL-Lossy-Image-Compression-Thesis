use clap::Parser;
use clap_derive::Parser;
use std::fs;

use benchmark_jpegxl::benchmark::{
    Benchmarker, /*CollectImageMetadataBenchmark,*/ JXLCompressionBenchmark,
};
use benchmark_jpegxl::config::Config;

/// Arguments
/// `--clean, -c` - Clean all benchmark files
/// `--temp, -t` - Use temp directory for benchmark files
/// `--libjxl_commit` - Use specific lbjxl commit or branch
/// `--compare_to_local` - Compare to local libjxl source
/// `--compare_to_commit` - Compare to specific libjxl commit or branch
#[derive(Parser)]
#[clap(name = "Benchmark JPEG-XL")]
struct Args {
    #[arg(short, long)]
    clean: bool,
    #[arg(short, long)]
    temp: bool,
    #[arg(long)]
    libjxl_commit: Option<String>,
    #[arg(long)]
    compare_to_local: bool,
    #[arg(long)]
    compare_to_commit: Option<String>,
}

/**
 * Main function for the JPEG-XL becnhmarker.
 */
fn main() {
    println!("Benchmark JPEG-XL");

    // Parse arguments.
    let args = Args::parse();

    // Set up config.
    // Use default config and add arguments.
    let mut config = Config::default();
    config.use_temp_dir = args.temp;
    config.libjxl_commit = args.libjxl_commit;
    config.compare_to_local = args.compare_to_local;
    config.compare_to_commit = args.compare_to_commit;

    // Set up benchmark directory.
    // Append "/temp" to benchmark directory if --temp is set.
    let benchmark_path = config.benchmark_dir_path.to_owned()
        + match args.temp {
            true => "/temp",
            false => match config.use_temp_dir {
                true => "/temp",
                false => "",
            },
        };
    config.benchmark_dir_path = benchmark_path.clone();

    // Clean benchmark directory if --clean is set.
    match args.clean {
        true => {
            println!("Cleaning benchmark directory at {}", benchmark_path);
            fs::remove_dir_all(benchmark_path.clone()).unwrap();
        }
        false => {}
    }

    // Create benchmark directory.
    println!("Creating benchmark directory at {}", benchmark_path);
    fs::create_dir_all(benchmark_path.clone()).unwrap();

    // Set up benchmarker.
    let mut benchmarker = Benchmarker::new(&config);

    //    println!("Running collect image metadata benchmark");
    //    let collect_image_metadata_benchmark = CollectImageMetadataBenchmark {};
    //    benchmarker.run_benchmark(&collect_image_metadata_benchmark);

    // Run JPEG-XL Compression benchmark.
    println!("Running JPEG-XL Compression benchmark");
    benchmarker.run_benchmark::<JXLCompressionBenchmark>();

    // Wait for workers to finish.
    benchmarker.wait_for_all_workers();

    // Teardown benchmarker.
//    benchmarker.teardown();
}
