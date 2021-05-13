#![no_std]
#![no_main]

use panic_halt as _;

use nb::block;

use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::{pac, prelude::*, timer::Timer, spi, serial};
use stm32f1xx_hal::dma::ReadWriteDma; // FIXME add this to the prelude
use core::fmt::Write;

static mut IN: [u8; 9] = [0; 9];
static mut IN2: [u8; 9] = [0; 9];
static OUT: [u8; 9] = [42, 1, 2, 3, 4, 5, 6, 7, 8];

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
	let txpin = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
	let rxpin = gpioa.pa10.into_floating_input(&mut gpioa.crh);

	let uart = serial::Serial::usart1(dp.USART1, (txpin, rxpin), &mut afio.mapr, serial::Config::default().baudrate(115200.bps()), clocks, &mut rcc.apb2);
	let (mut tx, rx) = uart.split();
	writeln!(tx, "Hello world!").unwrap();

	let mut spi = spi::Spi::spi1(dp.SPI1, (clk, miso, mosi), &mut afio.mapr, spi::Mode { phase: spi::Phase::CaptureOnFirstTransition, polarity: spi::Polarity::IdleLow }, 100.khz(), clocks, &mut rcc.apb2);

	let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

	let dma = dp.DMA1.split(&mut rcc.ahb);

	trigger.set_low().unwrap();
	let spi_dma = spi.with_rx_tx_dma(dma.2, dma.3);
	let fnord = spi_dma.read_write(unsafe { &mut IN }, &OUT);
	trigger.set_high().unwrap();
	let (_,spi_dma) = fnord.wait();
	trigger.set_low().unwrap();
	trigger.set_high().unwrap();

	spi_dma.read_write(unsafe { &mut IN2 }, &OUT).wait();

	writeln!(tx, "IN is {:?}", unsafe{IN}).unwrap();
	let first = unsafe{IN[0]};
	let yay = unsafe { IN[0] == 42 };

	//spi_dma.channel(


	/*let spi_dma = spi.with_tx_dma(dma.3);

	trigger.set_low().unwrap();
	let fnord = spi_dma.write(&spi_bytes);
	trigger.set_high().unwrap();
	fnord.wait();
	trigger.set_low().unwrap();
	trigger.set_high().unwrap();
	*/

	let mut timer = Timer::syst(cp.SYST, &clocks).start_count_down(if yay {1.hz()} else {3.hz()});
	loop {
		block!(timer.wait()).unwrap();
		led.set_high().unwrap();
		block!(timer.wait()).unwrap();
		led.set_low().unwrap();
	}
}
