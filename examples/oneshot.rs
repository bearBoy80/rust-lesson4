use std::{
    ops::Deref,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
    thread,
};

use anyhow::anyhow;
use arc_swap::ArcSwapOption;

struct OneShot<T> {
    data: ArcSwapOption<T>,
    state: AtomicU8, // 0 = empty, 1 = filled, 2 = consumed
}
struct Sender<T>(Arc<OneShot<T>>);
struct Receiver<T>(Arc<OneShot<T>>);

impl<T> OneShot<T> {
    fn channel() -> (Sender<T>, Receiver<T>) {
        let one_shot = Arc::new(OneShot {
            data: ArcSwapOption::from(None),
            state: AtomicU8::new(0),
        });
        (Sender(one_shot.clone()), Receiver(one_shot))
    }
}
impl<T> Receiver<T> {
    fn recv(&self) -> anyhow::Result<T> {
        if self
            .state
            .compare_exchange(1, 0, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            let v = self.data.swap(None);
            v.ok_or_else(|| anyhow!("no value"))
                .and_then(|v| Arc::try_unwrap(v).map_err(|_| anyhow!("Multiple receivers")))
        } else {
            Err(anyhow::anyhow!("Sender has not sent a value yet"))
        }
    }
}
impl<T> Sender<T> {
    fn send(&self, value: T) -> anyhow::Result<(), T> {
        if self
            .state
            .compare_exchange(1, 2, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            self.data.store(Some(Arc::new(value)));
            Ok(())
        } else {
            Err(value)
        }
    }
}
impl<T> Deref for Receiver<T> {
    type Target = Arc<OneShot<T>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> Deref for Sender<T> {
    type Target = Arc<OneShot<T>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
fn main() {
    let (tx, rx) = OneShot::channel();
    let sender = thread::spawn(move || {
        let value = 42;
        println!("sender value: {:?}", value);
        let _ = tx.send(value);
    });
    let receiver = thread::spawn(move || {
        let value = rx.recv();
        println!("receiver value: {:?}", value);
    });
    sender.join().unwrap();
    receiver.join().unwrap();
}
