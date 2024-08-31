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

    // 调试中断`int3`指令
    // x86_64::instructions::interrupts::int3();

    // 触发`page fault`
    // unsafe {
    //     *(0xdeadbeef as *mut u8) = 42;
    // }

    // 触发栈溢出
    // #[allow(unconditional_recursion)]
    // fn stack_overflow() {
    //     stack_overflow();
    // }
    // stack_overflow();

    // 触发死锁dead lock
    // loop {
    //     use rust_os::print;
    //     print!("-");
    //     for _ in 0..10000 {}
    // }

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
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
    rust_os::test_panic_handler(info)
}
