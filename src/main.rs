#![no_std] // 不链接 Rust 标准库
#![no_main] // 禁用所有 Rust 层级的入口点
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rust_os::{print, println};

// change `entry point` from `_start` to `kernel_main`
entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
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

    // page fault 1: 内存越界访问
    // let ptr = 0xdeadbeaf as *mut u8;
    // unsafe {
    //     // 写操作失败
    //     *ptr = 42;
    //     // 读操作失败
    //     // let _x = *ptr;
    //     // println!("read worked");
    // }

    // page fault 2: 权限错误, 读操作成功, 写操作失败
    // let ptr = 0x20426e as *mut u8;
    // unsafe {
    //     // 写操作失败
    //     *ptr = 42;
    //     println!("write worked");
    //     // 读操作成功
    //     // let _x = *ptr;
    //     // println!("read worked");
    // }

    // 内核中页表的存储方式
    // use x86_64::registers::control::Cr3;

    // let (level_4_page_table, _) = Cr3::read();
    // println!(
    //     "Level 4 page table at: {:?}",
    //     level_4_page_table.start_address()
    // );

    // print non-empty level 4 page table entries
    // print_l43_page_table(boot_info);

    // 测试地址翻译功能
    use rust_os::memory::translate_addr;
    use x86_64::VirtAddr;

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = unsafe { translate_addr(virt, phys_mem_offset) };
        println!("{:?} -> {:?}", virt, phys);
    }

    #[cfg(test)]
    test_main();

    print!("It did not crash");
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

#[allow(warnings)]
fn print_l43_page_table(boot_info: &'static BootInfo) {
    use rust_os::memory::active_level_4_table;
    use x86_64::structures::paging::PageTable;
    use x86_64::VirtAddr;

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let l4_table = unsafe { active_level_4_table(phys_mem_offset) };

    for (i, entry) in l4_table.iter().enumerate() {
        if !entry.is_unused() {
            println!("L4 Entry {}: {:?}", i, entry);

            // print non-empty the level 3 table entries
            let phys = entry.frame().unwrap().start_address();
            let virt = phys.as_u64() + boot_info.physical_memory_offset;
            let ptr = VirtAddr::new(virt).as_mut_ptr();
            let l3_table: &PageTable = unsafe { &*ptr };

            let mut count: usize = 0;
            for (_i, entry) in l3_table.iter().enumerate() {
                if !entry.is_unused() {
                    count += 1;
                }
            }
            println!("{} L3 entries", count);
        }
    }
}
