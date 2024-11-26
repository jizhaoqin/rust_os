#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rust_os::allocator::ALLOCATOR;
use rust_os::allocator::HEAP_SIZE;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    use rust_os::allocator;
    use rust_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    // 全局描述符表, 中断描述符表, 中断控制器初始化
    rust_os::init();

    // 动态内存(堆内存)分配器初始化
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}

#[test_case]
fn simple_allocation() {
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);
}

#[test_case]
fn large_vec() {
    let loops = 2000;
    let mut vec = Vec::new();
    for i in 0..loops {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (loops - 1) * loops / 2);
}

#[test_case]
fn many_boxes() {
    let plus = 1000;
    for i in 0..(HEAP_SIZE + plus) {
        let x = Box::new(i);

        assert_eq!(*x, i);
        assert_eq!(ALLOCATOR.lock().get_allocations(), 1);
    }
}

#[test_case]
fn many_boxes_long_lived() {
    let long_lived = Box::new(1);
    for i in 0..(HEAP_SIZE / 8 - 8) {
        let x = Box::new(i);
        assert_eq!(*x, i);
        assert_eq!(ALLOCATOR.lock().get_allocations(), 2);
    }
    assert_eq!(*long_lived, 1);
}
