#![no_std]
#![no_main]

mod epd;
use epd::*;
mod epd_draw_target;
use epd_draw_target::*;

use embedded_graphics::prelude::*;
use embedded_graphics::geometry::AnchorPoint;
use embedded_graphics::primitives::{Circle, Rectangle, Triangle, PrimitiveStyle, PrimitiveStyleBuilder, StrokeAlignment};
use embedded_graphics::text::{Alignment, Text};
use embedded_graphics::mono_font::{MonoTextStyle, ascii::FONT_10X20};

use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::delay::Delay;
use stm32f1xx_hal::pac;
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::spi::{self, Spi};

#[allow(unused_imports)]
use panic_halt;

// PA2: RST
// PA3: BUSY
// PA4: ECS
// PA5: SCK
// PA6: D/C
// PA7: MOSI

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    let mut flash = dp.FLASH.constrain();
    let clocks = rcc.cfgr.sysclk(8.mhz()).freeze(&mut flash.acr);

    let mut delay = Delay::new(cp.SYST, clocks);

    let mut led = gpiob.pb12.into_push_pull_output(&mut gpiob.crh);
    let mut epd = {
        let spi_pins = (
            gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl),
            spi::NoMiso,
            gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl),
        );
        let spi_mode = spi::Mode {
            polarity: spi::Polarity::IdleLow,
            phase: spi::Phase::CaptureOnFirstTransition,
        };
        let spi = Spi::spi1(
            dp.SPI1,
            spi_pins,
            &mut afio.mapr,
            spi_mode,
            100.khz(),
            clocks,
            &mut rcc.apb2,
        );
        let rst = gpioa.pa2.into_push_pull_output(&mut gpioa.crl);
        let busy = gpioa.pa3.into_floating_input(&mut gpioa.crl);
        let ecs = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);
        let dc = gpioa.pa6.into_push_pull_output(&mut gpioa.crl);

        Epd::new(spi, rst, busy, ecs, dc, &mut delay)
    };

    let mut display = epd;

    let thin_stroke = PrimitiveStyle::with_stroke(EpdColor::Black, 1);
    let thick_stroke = PrimitiveStyle::with_stroke(EpdColor::Black, 3);
    let border_stroke = PrimitiveStyleBuilder::new()
        .stroke_color(EpdColor::Red)
        .stroke_width(3)
        .stroke_alignment(StrokeAlignment::Inside)
        .build();
    let fill = PrimitiveStyle::with_fill(EpdColor::Black);
    let character_style = MonoTextStyle::new(&FONT_10X20, EpdColor::Red);

    let yoffset = 10;

    // Draw a 3px wide outline around the display.
    display
        .bounding_box()
        .resized(display.bounding_box().size - Size::new(4, 4), AnchorPoint::Center)
        .into_styled(border_stroke)
        .draw(&mut display).ok();

    // Draw a triangle.
    Triangle::new(
        Point::new(16, 16 + yoffset),
        Point::new(16 + 16, 16 + yoffset),
        Point::new(16 + 8, yoffset),
    )
    .into_styled(thin_stroke)
    .draw(&mut display).ok();

    // Draw a filled square
    Rectangle::new(Point::new(52, yoffset), Size::new(16, 16))
        .into_styled(fill)
        .draw(&mut display).ok();

    // Draw a circle with a 3px wide stroke.
    Circle::new(Point::new(88, yoffset), 17)
        .into_styled(thick_stroke)
        .draw(&mut display).ok();

    // Draw centered text.
    let text = "embedded\ngraphics";
    Text::with_alignment(
        text,
        display.bounding_box().center() + Point::new(0, 15),
        character_style,
        Alignment::Center,
    )
    .draw(&mut display).ok();

    led.set_high().ok();
    display.refresh();
    led.set_low().ok();

    loop {}
}
