/*!    ___     __ ______ ____  ___
      /  /__  / // __  //    \/  /
     /  // /_/ // /_/ //  /\    /
    /__//_____//_____//__/  \__/

    In this crate, you can find the target-side instrumentaions
    to annotate your beloved target.
*/

use core::ptr::{write_bytes, write_volatile};
pub use xxhash_rust::const_xxh3::xxh3_64 as const_xxh3;

/// Map size for easier declaration, default: 64KB
pub const MAP_SIZE: usize = 65_536;

/// Map for tracking, currently declared, but
/// the goal is to move this to the fuzzer side
#[unsafe(no_mangle)]
pub static mut MAP: [u8; MAP_SIZE] = [0; MAP_SIZE];

/// Pointer for accessing the map
#[unsafe(no_mangle)]
pub static mut MAP_PTR: *mut u8 = &raw mut MAP as _;

/// For zero the map
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ijon_zero_map() {
    unsafe {
        if !MAP_PTR.is_null() {
            write_bytes(MAP_PTR, 0, MAP_SIZE);
        }
    }
}

/// Follows the original IJON approach,
/// XOR the location hash with value to create unique coverage point
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ijon_set(loc_addr: u64, val: u64) {
    let combined: u64 = loc_addr ^ val;
    let idx: usize = (combined % (MAP_SIZE as u64)) as usize;
    unsafe { write_volatile(MAP_PTR.add(idx), 1) };
    concat!();
}

/// Mark the value `m` as a new coverage event.
/// Each distinct value of `m` will be treated like a new branch.
/// Example: `ijon_set!((x, y))` rewards visiting new (x,y) positions in a maze.
#[macro_export]
macro_rules! ijon_set {
    ($m:expr) => {
        unsafe {
            if !$crate::MAP_PTR.is_null() {
                // trying to get better performance by hashing
                // the filename and line number at compile time
                const LOC_CACHE: u64 = $crate::const_xxh3(concat!(file!(), line!()).as_bytes());

                let val: u64 = ::libafl_bolts::generic_hash_std(&$m);

                ::libafl_ijon::ijon_set(LOC_CACHE, val);
            }
        }
    };
}
