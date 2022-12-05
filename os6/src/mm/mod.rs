mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_tale;

pub use address::*;
pub use frame_allocator::*;
pub use memory_set::*;
pub use page_tale::translated_byte_buffer;
pub use page_tale::translated_refmut;
pub use page_tale::translated_str;
pub use page_tale::PageTable;
pub use page_tale::PageTableEntry;
pub use page_tale::UserBuffer;

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
}
