
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate zstd;

use std::env;
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;

type XCheckWriter = zstd::stream::Encoder<File>;

lazy_static! {
    static ref RB_XCHECK_MUTEX: Mutex<Option<XCheckWriter>> = {
        extern fn cleanup() {
            // Flush and close the file on program exit
            let mut guard = RB_XCHECK_MUTEX.lock().unwrap();
            let out = guard.take().unwrap();
            out.finish().expect("Failed to finish encoding");
        }
        unsafe { libc::atexit(cleanup) };

        let xchecks_file = env::var("CROSS_CHECKS_OUTPUT_FILE")
            .expect("Expected file path in CROSS_CHECKS_OUTPUT_FILE variable");
        let file = File::create(xchecks_file.clone())
            .unwrap_or_else(|e| panic!("Failed to create cross-checks log file {}: {}", xchecks_file, e));
        let encoder = zstd::stream::Encoder::new(file, 0)
            .expect("Failed to create zstd encoder");
        Mutex::new(Some(encoder))
    };
}

#[no_mangle]
pub extern "C" fn rb_xcheck(tag: u8, val: u64) {
    let mut guard = RB_XCHECK_MUTEX.lock().unwrap();
    let out = guard.as_mut().unwrap();
    out.write_all(&[tag]).expect("Failed to write tag");
    out.write_all(&val.to_le_bytes())
        .expect("Failed to write value");
}
