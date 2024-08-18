#![no_std] // 不链接 Rust 标准库
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rust_os::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 链接器会寻找一个默认名为 `_start` 的函数，所以这个函数就是入口点
    println!("Hello World{}", "!");

    rust_os::init();
    x86_64::instructions::interrupts::int3();

    #[cfg(test)]
    test_main();

    println!("It dit not crash!");
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}
