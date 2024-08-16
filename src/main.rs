#![no_std] // 不链接 Rust 标准库
#![no_main] // 禁用所有 Rust 层级的入口点
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rust_os::println;

#[no_mangle] // 不重整函数名
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    rust_os::init();

    // 调试中断
    // x86_64::instructions::interrupts::int3();

    // page fault
    // unsafe {
    //     *(0xdeadbeef as *mut u8) = 42;
    // };

    // stack overflow
    // #[allow(unconditional_recursion)]
    // fn stack_overflow() {
    //     stack_overflow();
    // }
    // stack_overflow();

    // 死锁
    // loop {
    //     use rust_os::print;

    //     for _ in 0..10000 {}
    //     print!("-");
    // }

    #[cfg(test)]
    test_main();

    println!("It did not crash");
    rust_os::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    rust_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info);
}
