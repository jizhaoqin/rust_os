use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

/// 初始化一个新的OffsetPageTable。
///
/// 这个函数是不安全的，因为调用者必须保证完整的物理内存在
/// 传递的`physical_memory_offset`处被映射到虚拟内存。另
/// 外，这个函数必须只被调用一次，以避免别名"&mut "引用（这是未定义的行为）。
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// 返回一个对活动的4级表的可变引用。
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

/// 为给定的页面创建一个实例映射到框架`0xb8000`。
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        // FIXME: 这并不安全，我们这样做只是为了测试。
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}

/// 一个总是返回`None'的FrameAllocator。
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}
