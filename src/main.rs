#![no_std] // 不链接 Rust 标准库
#![no_main] // 禁用所有 Rust 层级的入口点
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rust_os::{print, println};

// change `entry point` from `_start` to `kernel_main`
entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello World{}", "!");

    // 内核初始化
    rust_os::init(boot_info);

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
    // 使用 OffsetPageTable
    // test_address_translate(boot_info);

    // 创建一个新的映射
    // test_create_new_map(boot_info);

    // try execute async tasks
    use rust_os::task::{simple_executor::SimpleExecutor, Task};

    let mut executor = SimpleExecutor::new();
    executor.spawn(Task::new(example_task()));
    executor.run();

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

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

#[test_case]
fn test_heap_allocation() {
    use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};

    // allocate a number on the heap
    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    // create a dynamically sized vector
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!(
        "vec at {:p} with capacity {}",
        // &vec,
        &vec[..],
        // vec.as_slice(),
        vec.capacity()
    );

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!(
        "current reference count is {}",
        Rc::strong_count(&cloned_reference)
    );
    core::mem::drop(reference_counted);
    println!(
        "reference count is {} now",
        Rc::strong_count(&cloned_reference)
    );
}

#[allow(dead_code)]
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

#[allow(dead_code)]
fn test_address_translate(boot_info: &BootInfo) {
    use rust_os::memory;
    use x86_64::{structures::paging::Translate, VirtAddr};

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mapper = unsafe { memory::init(phys_mem_offset) };

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
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
    }
}

#[allow(dead_code)]
fn test_create_new_map(boot_info: &'static BootInfo) {
    use rust_os::memory;
    use x86_64::{structures::paging::Page, VirtAddr};
    // 新的导入

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    // let mut frame_allocator = memory::EmptyFrameAllocator;
    // 创建丢失的页表
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // 映射未使用的页
    // let page = Page::containing_address(VirtAddr::new(0));
    // 映射一个还不存在一级表的页面
    let page: Page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // 通过新的映射将字符串 `New!`  写到屏幕上。
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };
}
