use crate::*;
use crate::subscriber::Subscriber;

pub type SubscriberMap = HashMap<Vec<Arc<Uuid>>, Subscriber>;