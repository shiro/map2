use crate::*;

pub fn get_id_and_incremen(counter: &AtomicU32) -> u32 {
    counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
}
