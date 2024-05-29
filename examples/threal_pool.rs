use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

fn main() {
    let pool = ThreadPool::new(2);
    pool.execute(|| println!("{:?}: 执行", thread::current().id()));
    thread::sleep(Duration::from_secs(3));
}
struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}
struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}
impl Worker {
    fn new(id: usize, reciver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            while let Ok(job) = reciver.lock().unwrap().recv() {
                println!("worker-{} starting do job", id);
                job();
            }
        });
        Worker { id, thread }
    }
}
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    fn new(size: usize) -> Self {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let mut workers = Vec::with_capacity(size);
        let receiver = Arc::new(Mutex::new(receiver));
        for id in 0..size {
            workers.push(Worker::new(id, receiver.clone()));
        }
        ThreadPool { workers, sender }
    }
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job);
    }
}
