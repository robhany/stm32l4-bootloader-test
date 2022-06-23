#![no_main]
#![no_std]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate panic_semihosting;
extern crate stm32l4xx_hal;

use rt::{entry, ExceptionFrame};

const FLASH_USER: u32 = 0x0801_0000;

#[entry]
fn main() -> ! {
    //check if application is present
    let sp = unsafe { (FLASH_USER as *const u32).read() };
    if sp & 0xfffe_0000 == 0x2000_0000 {
        let vt = FLASH_USER as *const u32;
        unsafe {
            cortex_m::asm::bootload(vt);
        }
    }

    panic!("No application found")
}

#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
