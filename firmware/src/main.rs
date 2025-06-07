use usb_device::prelude::*;
use usbd_hid::descriptor::KeyboardReport;
use usbd_hid::hid_class::HIDClass;
use usbd_msd::BlockDevice;
use usbd_msd::UsbMassStorage;

fn main() {
    let usb_bus =;
    let mut hid = HIDClass::new(&usb_bus, KeyboardReport::desc(), 10);
    let mut storage = UsbMassStorage::new(&usb_bus, BlockDevice::new());
    let mut dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1209, 0x0001))
        .manufacturer("Acme")
        .product("Ninja Charger")
        .build();

    loop {
        dev.poll(&mut [&mut hid, &mut storage]);
        let report = KeyboardReport { modifier: 0, leds: 0, keycodes: [0; 6] };
        hid.push_input(&report).ok();
    }
}
