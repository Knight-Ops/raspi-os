#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![feature(trait_alias)]
#![no_main]
#![no_std]

mod bsp;
mod interface;
mod print;
mod runtime_init;

fn kernel_entry() -> ! {
    use interface::console::Statistics;
    for i in bsp::device_drivers().iter() {
        if let Err(()) = i.init() {
            panic!("Error loading driver: {}", i.compatible())
        }
    }

    println!("[0] Hello from pure Rust!");

    println!("[1] Drivers probed:");
    for (i, driver) in bsp::device_drivers().iter().enumerate() {
        println!("    {}. {}", i + 1, driver.compatible());
    }

    println!(
        "[2] Characters written : {}",
        bsp::console().chars_written()
    );

    panic!("Stopping at end of kernel_entry");
}
