use std::sync::atomic::{self, AtomicU32, Ordering};

use serde::{Deserialize, Serialize};

static CURRENT_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
/// Nondecreasing id generator for unique generation of ids for locating tasks
pub struct TaskId(u32);

impl TaskId {
    /// Returns the internal value stored in the TaskId
    pub fn as_u32(&self) -> u32 {
        self.0
    }

    /// Returns the next TaskId
    pub fn new_unique_id() -> TaskId {
        TaskId(CURRENT_ID.fetch_add(1, atomic::Ordering::Relaxed))
    }

    /// If a TaskId was generated in a different run of the program, to ensure ids are unique, the current id can be
    /// increased if needed
    pub fn set_if_greater(new_val: u32) {
        let _old = CURRENT_ID.fetch_max(new_val, Ordering::SeqCst);
    }
}
