use crate::mem_manager::pmm;
use x86_64::registers::control::Cr3;
use dog_essentials::static_cell::StaticCell;

static k_page_table: StaticCell<u64> = StaticCell::new(0);

pub fn init(mem_map_size: u32, page_table_size: u32) {
    unsafe {
        k_page_table.set_value_unsafe(Cr3::read().0.start_address().as_u64());
        //let x: *mut PageTable = unsafe { *k_page_table.get_value_unsafe() as *mut u8 as *mut PageTable };
    }
    
    pmm::init_from_mem_map(mem_map_size);
}
