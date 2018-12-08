use log::info;
use logging_allocator::{run_guarded, LoggingAllocator};

#[global_allocator]
static ALLOC: LoggingAllocator = LoggingAllocator::new();

fn main() {
    simple_logger::init().unwrap();

    ALLOC.enable_logging();
    run_guarded(|| info!("Creating an empty vector"));
    let mut vec = vec![0; 4];
    run_guarded(|| info!("Inserting some numbers"));
    vec.extend(&[1, 2, 3, 4, 5]);
    run_guarded(|| info!("Cloning the vector"));
    let _clone = vec.clone();
    run_guarded(|| info!("Dropping the original vector"));
    drop(vec);
    run_guarded(|| info!("Finished!"));
    ALLOC.disable_logging();
}
