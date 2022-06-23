#![no_main]
#![no_std]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate cortex_m_semihosting;

extern crate panic_semihosting;
extern crate stm32l4xx_hal as hal;

use core::{
    cell::RefCell,
    fmt::Write,
    ops::DerefMut,
    sync::atomic::{AtomicU32, Ordering},
};

use cortex_m::{
    interrupt::{free, Mutex},
    peripheral::NVIC,
};
use cortex_m_semihosting::hio::hstdout;
use hal::{
    delay::Delay,
    interrupt,
    prelude::*,
    timer::{Event, Timer},
};

use rt::{entry, ExceptionFrame};

static COUNT: AtomicU32 = AtomicU32::new(0);
static TIMER_TIM7: Mutex<RefCell<Option<Timer<hal::stm32::TIM7>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let mut hstdout = hstdout().unwrap();
    writeln!(hstdout, "Hello from main").unwrap();
    let cortex_peripherals = cortex_m::Peripherals::take().unwrap();
    let hal_peripherals = hal::stm32::Peripherals::take().expect("failed to get stm32 peripherals");
    let mut flash = hal_peripherals.FLASH.constrain();
    let mut rcc = hal_peripherals.RCC.constrain();
    let mut pwr = hal_peripherals.PWR.constrain(&mut rcc.apb1r1);
    let clocks = rcc.cfgr.freeze(&mut flash.acr, &mut pwr);
    let mut delay = Delay::new(cortex_peripherals.SYST, clocks);
    unsafe { NVIC::unmask(hal::stm32::Interrupt::TIM7) };
    let mut timer = Timer::tim7(hal_peripherals.TIM7, 1.Hz(), clocks, &mut rcc.apb1r1);
    timer.listen(Event::TimeOut);
    free(|cs| {
        TIMER_TIM7.borrow(cs).replace(Some(timer));
    });

    writeln!(hstdout, "Timer init done!").unwrap();

    let mut last_count = COUNT.load(Ordering::Relaxed);
    loop {
        writeln!(hstdout, "loop").unwrap();
        delay.delay_ms(100_u32);
        let count = COUNT.load(Ordering::Relaxed);
        if last_count != count {
            last_count = count;
            writeln!(hstdout, "count changed").unwrap();
        }
    }
}

#[interrupt]
fn TIM7() {
    let mut hstdout = hstdout().unwrap();
    writeln!(hstdout, "Hello, from TIMER").unwrap();
    free(|cs| {
        if let Some(ref mut tim7) = TIMER_TIM7.borrow(cs).borrow_mut().deref_mut() {
            tim7.clear_interrupt(Event::TimeOut);
        }
    });
    COUNT.fetch_add(1, Ordering::Relaxed);
}

#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
