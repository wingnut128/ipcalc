#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = ipcalc::Ipv4Subnet::from_cidr(s);
        let _ = ipcalc::Ipv6Subnet::from_cidr(s);
    }
});
