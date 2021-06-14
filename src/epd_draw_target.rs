use embedded_graphics_core::pixelcolor::raw::RawU2;
use embedded_graphics_core::prelude::*;

use crate::epd::*;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum EpdColor {
    White,
    Black,
    Red,
}

impl EpdColor {
    fn black_bit(&self) -> u8 {
        match self {
            EpdColor::Black => 0,
            _ => 1,
        }
    }

    fn red_bit(&self) -> u8 {
        match self {
            EpdColor::Red => 0,
            _ => 1,
        }
    }
}

impl PixelColor for EpdColor {
    type Raw = RawU2;
}

impl From<RawU2> for EpdColor {
    fn from(data: RawU2) -> Self {
        match data.into_inner() {
            0b00 => Self::White,
            0b01 => Self::Black,
            0b10 => Self::Red,
            0b11 => panic!("unused RawU2 value 0b11"),
            _ => unreachable!(),
        }
    }
}

impl<SPI, RST, BUSY, ECS, DC> OriginDimensions for Epd<SPI, RST, BUSY, ECS, DC> {
    fn size(&self) -> Size {
        Size::new(152, 152)
    }
}

impl<SPI, RST, BUSY, ECS, DC> DrawTarget for Epd<SPI, RST, BUSY, ECS, DC> {
    type Color = EpdColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            let Point { x, y } = coord;
            let position = y * 152 + x;
            let index = (position / 8) as usize;
            let bit = 7 - position % 8;

            self.framebuffer_black[index] &= !(1 << bit);
            self.framebuffer_black[index] |= color.black_bit() << bit;

            self.framebuffer_red[index] &= !(1 << bit);
            self.framebuffer_red[index] |= color.red_bit() << bit;
        }

        Ok(())
    }
}
