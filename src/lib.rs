use std::{thread, sync::{mpsc::{self}, Arc, Mutex}};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate
}

impl ThreadPool {
    /**
     * Create new threadpool based on numbers of threads.
     */
    pub fn new(size: usize) -> Self {

        let mut workers = Vec::with_capacity(size);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)))
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where 
        F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            println!("Shutting down worker: {}", worker.id);   
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }

    }
}


struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl  Worker {
    pub fn new (id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker{
        let thread = thread::spawn(move || loop { 
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("Worker receive the job {}", id);
                    job();
                },
                Message::Terminate => {
                    println!("Worker {} are terminating", id);
                    break;
                }
            }
        });
        Worker { id, thread: Some(thread) }
    }
}
