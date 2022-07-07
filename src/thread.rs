use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct ThreadPool {
    workers: Vec<Worker>,
    tx: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// Create a new thead pool.
    ///
    /// # Panics
    ///
    /// Panics if trying to create a thread pool with 0 or less threads.
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let (tx, rx) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&rx)));
        }

        Self { tx, workers }
    }

    pub fn execute<T>(&self, t: T)
    where
        T: FnOnce() + Send + 'static,
    {
        let job = Box::new(t);

        self.tx.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        eprintln!("Sending terminate message to all worker threads!");

        for worker in &self.workers {
            eprintln!("Worker {} told to terminate!", worker.id);
            self.tx.send(Message::Terminate).unwrap();
        }

        eprintln!("Shutting down all workers!");

        for worker in &mut self.workers {
            eprintln!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap()
            };
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, rx: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let message = rx.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    eprintln!("Worker {}, got a job!", id);
                    job();
                }
                Message::Terminate => {
                    eprintln!("Worker {}, told to terminate!", id);
                    break;
                }
            }
        });

        Self {
            id,
            thread: Some(thread),
        }
    }
}
