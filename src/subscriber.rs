use crate::*;
use crate::mapper::MapperInner;
use crate::writer::WriterInner;


pub enum Subscriber {
    Mapper(Arc<MapperInner>),
    Writer(Arc<WriterInner>)
}

impl Subscriber {
    pub fn handle(&self, id: &str, ev: InputEvent) {
        match self {
            Subscriber::Mapper(target) => { target.handle(id, ev) }
            Subscriber::Writer(target) => { target.handle(id, ev) }
        }
    }
}
