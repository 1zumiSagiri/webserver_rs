use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

// wrap thread to allow for graceful dropping
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        // let closure take over the receiver
        // and loop until the receiver is closed
        let thread = thread::spawn(move || loop {
            let msg = receiver.lock().unwrap().recv();

            match msg {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();
                }

                Err(_) => {
                    println!("Worker {id} disconnected, shutting down.");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

// workers model:
// workers receive jobs from queue of jobs
pub struct ThreadPool {
    // refer to thread::spawn
    workers: Vec<Worker>,
    // channel to send jobs to workers
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        // ThreadPool owns the sending side of the channel
        // sending jobs to workers via execute()
        // Workers own the receiving side of the channel
        let (sender, receiver) = mpsc::channel();

        // make a thread safe reference to the receiver
        // so that multiple workers can share the receiver
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    // execute method that takes a closure and runs it on a thread
    // execute only needs to be called once
    // trait Send allows the closure to be sent to another thread
    // trait 'static means the closure does not have any references to the stack
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        // send the job to the worker
        // as_ref() returns a reference to the Option
        self.sender.as_ref().unwrap().send(job).unwrap()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // drop the sender to signal to workers to stop
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            // take the ownership of thread from Option
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap()
            }
        }
    }
}
