use crate::subscriber::SubscriberNew;

pub fn hash_path(path: &Vec<uuid::Uuid>) -> u64 {
    use std::hash::Hash;
    use std::hash::Hasher;
    let mut h = std::hash::DefaultHasher::new();
    path.hash(&mut h);
    let path_hash = h.finish();
    path_hash
}

pub trait Subscribable {
    fn subscribe(&self) -> SubscriberNew;
}
