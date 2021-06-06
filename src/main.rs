#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::{delay::Delay, pac, prelude::*};

#[allow(unused_imports)]
use panic_halt;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let mut led = gpiob.pb12.into_push_pull_output(&mut gpiob.crh);

    let mut flash = dp.FLASH.constrain();
    let clocks = rcc.cfgr.sysclk(8.mhz()).freeze(&mut flash.acr);

    let mut delay = Delay::new(cp.SYST, clocks);

    loop {
        led.set_high().ok();
        delay.delay_ms(1_000u16);
        led.set_low().ok();
        delay.delay_ms(1_000u16);
    }
}
