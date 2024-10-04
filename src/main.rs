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

    // Page Fault 1: 触发内存越界访问
    // let ptr = 0xdeadbeaf as *mut u8;
    // unsafe {
    //     // 写入失败
    //     // *ptr = 42;
    //     // 读取失败
    //     // let _x = *ptr;
    //     // println!("read worked");
    // }

    // Page Fault 2: 非法写入
    // let ptr = 0x20426c as *mut u8;
    // unsafe {
    //     // 读取成功
    //     let _x = *ptr;
    //     println!("read worked");
    //     // 写入失败
    //     *ptr = 42;
    //     println!("write worked");
    // }

    // 内核中页表的存储方式
    use x86_64::registers::control::Cr3;
    let (level_4_page_table, _) = Cr3::read();
    println!(
        "Level 4 page table at: {:?}",
        level_4_page_table.start_address()
    );

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
