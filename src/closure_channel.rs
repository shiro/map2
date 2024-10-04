use anyhow::{anyhow, Error, Result};
use futures_time::prelude::*;
use tokio::sync::mpsc::{channel, Receiver, Sender};

pub struct ClosureChannel<Value> {
    tx: Sender<Box<dyn FnOnce(&mut Value) + Send>>,
}

impl<V> Clone for ClosureChannel<V> {
    fn clone(&self) -> Self {
        Self { tx: self.tx.clone() }
    }
}

impl<Value: 'static> ClosureChannel<Value> {
    pub fn new() -> (Self, Receiver<Box<dyn FnOnce(&mut Value) + Send>>) {
        let (mut tx, mut rx) = channel(64);
        (Self { tx }, rx)
    }

    pub fn call<'a, Ret: Send + 'a>(&self, closure: Box<dyn FnOnce(&mut Value) -> Ret + Send + 'a>) -> Result<Ret> {
        futures::executor::block_on(self.call_async(closure))
    }

    pub async fn call_async<'a, Ret: Send + 'a>(
        &self,
        closure: Box<dyn FnOnce(&mut Value) -> Ret + Send + 'a>,
    ) -> Result<Ret> {
        let (mut tx, mut rx) = tokio::sync::oneshot::channel::<Ret>();

        let cb = Box::new(move |value: &mut Value| {
            let ret = closure(value);
            tx.send(ret).map_err(|err| anyhow!("failed to send return message")).unwrap();
        });

        // we guarantee the lifetimes are compatible
        let cb = unsafe {
            std::mem::transmute::<Box<dyn FnOnce(&mut Value) + Send + 'a>, Box<dyn FnOnce(&mut Value) + Send + 'static>>(
                cb,
            )
        };

        self.tx.try_send(cb).map_err(|err| anyhow!("closure channel error: failed to send message"))?;

        match rx.timeout(futures_time::time::Duration::from_millis(5000)).await {
            Ok(ret) => match ret {
                Ok(ret) => Ok(ret),
                Err(err) => Err(anyhow!("closure channel error: other side already closed")),
            },
            Err(_) => Err(anyhow!("closure channel timed out, probably due to a deadlock")),
        }
    }
}
