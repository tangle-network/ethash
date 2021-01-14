use std::io::prelude::*;

fn main() {
    // a poor man cli parser.
    let mut args = std::env::args().skip(1);

    let from: u32 = args
        .next()
        .map(|v| v.parse().unwrap_or_else(|_| print_help()))
        .unwrap_or_else(|| print_help());
    let to: u32 = args
        .next()
        .map(|v| v.parse().unwrap_or_else(|_| print_help()))
        .unwrap_or_else(|| print_help());

    println!(
        "Starting to calculate DAG Roots from {} to {} (i.e {} epochs)",
        from,
        to,
        to.wrapping_sub(from)
    );

    let mut roots = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .truncate(false)
        .open("roots.txt")
        .expect("Failed to create/open roots.txt file");

    for i in from..to {
        let epoch = i as usize;
        let dataset_size = ethash::get_full_size(epoch);
        let mut dataset = vec![0u8; dataset_size];
        println!(
            "epoch {} dataset size: {} MB",
            epoch,
            dataset_size / (1024 * 1024)
        );
        let dag = ethash::LightDAG::<ethash::EthereumPatch>::new(epoch.into());
        let cache = dag.cache;
        ethash::make_dataset(&mut dataset, &cache);
        let root = ethash::calc_dataset_merkle_root(epoch, &dataset);
        println!("{}:{:?}", epoch, root);
        writeln!(roots, "{}:{:?}", epoch, root)
            .expect("failed to save the root for the last epoch");
    }
}

fn print_help() -> ! {
    println!("usage: epoch <FROM_EPOCH> <TO_EPOCH>");
    std::process::exit(1);
}
