use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::{InputPin, OutputPin};

struct EpdInterface<SPI, RST, BUSY, ECS, DC> {
    spi: SPI,
    rst: RST,
    busy: BUSY,
    ecs: ECS,
    dc: DC,
}

impl<SPI, RST, BUSY, ECS, DC> EpdInterface<SPI, RST, BUSY, ECS, DC>
where
    SPI: spi::Write<u8>,
    RST: OutputPin,
    BUSY: InputPin,
    ECS: OutputPin,
    DC: OutputPin,
{
    const CMD_PSR: u8 = 0x00;
    const CMD_PWR: u8 = 0x01;
    const CMD_PON: u8 = 0x04;
    const CMD_BTST: u8 = 0x06;
    const CMD_DTM1: u8 = 0x10;
    const CMD_DRF: u8 = 0x12;
    const CMD_DTM2: u8 = 0x13;
    const CMD_PLL: u8 = 0x30;
    const CMD_CDI: u8 = 0x50;
    const CMD_TRES: u8 = 0x61;
    const CMD_VDCS: u8 = 0x82;

    const DISPLAY_WIDTH: usize = 152;
    const DISPLAY_HEIGHT: usize = 152;

    const DISPLAY_HORIZONTAL_BANKS: usize = (Self::DISPLAY_WIDTH + 7) / 8;
    const DISPLAY_VERTICAL_BANKS: usize = (Self::DISPLAY_HEIGHT + 7) / 8;

    const FRAME_BYTES: usize = Self::DISPLAY_WIDTH * Self::DISPLAY_HEIGHT / 8;

    pub fn new<DELAY: DelayMs<u8>>(
        spi: SPI,
        mut rst: RST,
        busy: BUSY,
        mut ecs: ECS,
        dc: DC,
        delay: &mut DELAY,
    ) -> Self {
        let mut sself = Self {
            spi,
            rst,
            busy,
            ecs,
            dc,
        };

        sself.reset(delay);

        sself.cmd(Self::CMD_PWR, &[0x03, 0x00, 0x2b, 0x2b, 0x09]);
        sself.cmd(Self::CMD_BTST, &[0x17, 0x17, 0x17]);
        sself.cmd(Self::CMD_PON, &[]);

        // T_{pwr_on} = 80ms from datasheet
        delay.delay_ms(100);

        sself.cmd(Self::CMD_PSR, &[0xcf]);
        sself.cmd(Self::CMD_CDI, &[0x37]);
        sself.cmd(Self::CMD_PLL, &[0x29]);
        sself.cmd(Self::CMD_VDCS, &[0x0a]);

        // Adafruit library has a bit of a delay here for some reason
        // (stabilising?)
        delay.delay_ms(10);

        let display_height_bytes = (Self::DISPLAY_HEIGHT as u16).to_be_bytes();
        sself.cmd(
            Self::CMD_TRES,
            &[
                Self::DISPLAY_WIDTH as u8,
                display_height_bytes[0],
                display_height_bytes[1],
            ],
        );

        sself
    }

    fn reset<DELAY: DelayMs<u8>>(&mut self, delay: &mut DELAY) {
        self.rst.set_low().ok();
        delay.delay_ms(100);
        self.rst.set_high().ok();
    }

    pub fn refresh(&mut self, black: &[u8; 2888], red: &[u8; 2888]) {
        self.cmd(Self::CMD_DTM1, black);
        self.cmd(Self::CMD_DTM2, red);
        self.cmd(Self::CMD_DRF, &[]);

        while self.busy.is_high().ok().unwrap() {}
    }

    fn cmd(&mut self, cmd: u8, bytes: &[u8]) {
        // select
        self.ecs.set_low().ok();

        // command phase
        self.dc.set_low().ok();
        self.spi.write(&[cmd]).ok();

        // data phase
        if bytes.len() != 0 {
            self.dc.set_high().ok();
            self.spi.write(bytes).ok();
        }

        // deselect
        self.ecs.set_high().ok();
    }
}

pub struct Epd<SPI, RST, BUSY, ECS, DC> {
    interface: EpdInterface<SPI, RST, BUSY, ECS, DC>,
    pub framebuffer_black: [u8; 2888],
    pub framebuffer_red: [u8; 2888],
}

impl<SPI, RST, BUSY, ECS, DC> Epd<SPI, RST, BUSY, ECS, DC>
where
    SPI: spi::Write<u8>,
    RST: OutputPin,
    BUSY: InputPin,
    ECS: OutputPin,
    DC: OutputPin,
{
    pub fn new<DELAY: DelayMs<u8>>(
        spi: SPI,
        mut rst: RST,
        busy: BUSY,
        mut ecs: ECS,
        dc: DC,
        delay: &mut DELAY,
    ) -> Self {
        Self {
            interface: EpdInterface::new(spi, rst, busy, ecs, dc, delay),
            framebuffer_black: [0xff; 2888],
            framebuffer_red: [0xff; 2888],
        }
    }

    pub fn refresh(&mut self) {
        self.interface
            .refresh(&self.framebuffer_black, &self.framebuffer_red);
    }
}
