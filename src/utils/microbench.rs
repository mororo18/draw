#[allow(dead_code)]
pub
struct MicroBench {
    start: u64,
}

#[allow(dead_code)]
impl MicroBench {

    fn now() -> Self {

        let stamp = Self::read_tsc();

         Self {
             start: stamp,
         }
    }

    fn elapsed(&self) -> u64 {
        let now = Self::read_tsc();
        return now - self.start;
    }


    fn read_tsc() -> u64 {
        use core::arch::x86_64::__rdtscp;

         let clock: u64;

         unsafe {
             let tmp: u32 = 0;
             let ptr: *const u32 = &tmp;
             clock = __rdtscp(ptr as *mut u32);
         };

         return clock;
    }
}
