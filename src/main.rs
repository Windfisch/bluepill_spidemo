#![deny(unsafe_code)]
#![no_std]
#![no_main]

use panic_halt as _;

use nb::block;

use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::{pac, prelude::*, timer::Timer, spi};

static spi_bytes: [u8; 9] = [42, 1, 2, 3, 4, 5, 6, 7, 8];

#[entry]
fn main() -> ! {
	let cp = cortex_m::Peripherals::take().unwrap();
	let dp = pac::Peripherals::take().unwrap();

	let mut flash = dp.FLASH.constrain();
	let mut rcc = dp.RCC.constrain();
	let clocks = rcc.cfgr.freeze(&mut flash.acr);

	let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

	let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
	let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

	let mut trigger = gpioa.pa4.into_push_pull_output_with_state(&mut gpioa.crl, stm32f1xx_hal::gpio::State::High);
	let clk = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
	let miso = gpioa.pa6.into_floating_input(&mut gpioa.crl);
	let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

	let mut spi = spi::Spi::spi1(dp.SPI1, (clk, miso, mosi), &mut afio.mapr, spi::Mode { phase: spi::Phase::CaptureOnFirstTransition, polarity: spi::Polarity::IdleLow }, 100.khz(), clocks, &mut rcc.apb2);

	let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
	let mut timer = Timer::syst(cp.SYST, &clocks).start_count_down(1.hz());

	let dma = dp.DMA1.split(&mut rcc.ahb);
	let spi_dma = spi.with_tx_dma(dma.3);

	trigger.set_low().unwrap();
	let fnord = spi_dma.write(&spi_bytes);
	trigger.set_high().unwrap();
	fnord.wait();
	trigger.set_low().unwrap();
	trigger.set_high().unwrap();

	loop {
		block!(timer.wait()).unwrap();
		led.set_high().unwrap();
		block!(timer.wait()).unwrap();
		led.set_low().unwrap();
	}
}
