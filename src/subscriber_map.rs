use crate::subscriber::Subscriber;
use crate::*;

// [path_hash] -> (next_path_hash, Subscriber)
pub type SubscriberMap = HashMap<u64, (u64, Subscriber)>;

