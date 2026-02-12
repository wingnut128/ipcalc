#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Some(idx) = s.find('\0') {
            let cidr = &s[..idx];
            let address = &s[idx + 1..];
            let _ = ipcalc::contains::check_ipv4_contains(cidr, address);
            let _ = ipcalc::contains::check_ipv6_contains(cidr, address);
        }
    }
});
