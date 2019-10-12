#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![feature(trait_alias)]
#![no_main]
#![no_std]

// These imports are essentially in the order of execution for the kernel
// First we have the _start() function which is architecture specific and should be able to stay the same across
// the same architecture
mod arch;

// Runtime init currently just zero's our BSS but then jumps to the kernel_entry
mod runtime_init;

// Our BSP will include code to support the specific board that we are using
mod bsp;

// This includes our kernel code for driver interfaces and panic behavior. If a panic happens prior to the kernel booting, we won't
// know about it, and it will try to call non-existant functions. This should be prevented by prior to this, error handling should
// be done with wait_forever()
mod interface;
mod panic_wait;
mod print;

fn kernel_entry() -> ! {
    use interface::console::All;
    bsp::init();

    loop {
        if bsp::console().read_char() == '\n' {
            break;
        }
    }

    println!("[0] Booting on <{}>", bsp::board_name());

    println!("[1] Drivers loaded:");
    for (i, driver) in bsp::device_drivers().iter().enumerate() {
        println!("    {}. {}", i + 1, driver.compatible());
    }

    println!(
        "[3] Characters written : {}",
        bsp::console().chars_written()
    );

    println!("[4] Echoing input now.");
    loop {
        let c = bsp::console().read_char();
        bsp::console().write_char(c);
    }

    panic!("Stopping at end of kernel_entry");
}
