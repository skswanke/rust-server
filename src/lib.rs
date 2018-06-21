use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

pub enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, reciever) = mpsc::channel();

        let reciever = Arc::new(Mutex::new(reciever));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&reciever)));
        }

        ThreadPool {
            workers,
            sender,
        }
    }
    
    /// Execute closure on ThreadPool.
    ///
    /// The f is the clocure to execute on the threadpool.
    ///
    /// # Panics
    ///
    /// The `execute` function will panic if the channel is unable to take the job.
    pub fn execute<F>(&self, f: F)
        where F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Terminate message sent");;

        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            println!("Worker {} : Shutting down", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Create a new Worker.
    ///
    /// The id is the id of the worker.
    /// The reciever is a reciever for the channel which provides jobs to the worker.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if another worker panics while holding the mutex.
    /// The `new` function will panic if the recieved job panics.
    pub fn new(id: usize, reciever: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = reciever.lock().unwrap().recv().unwrap();

                match message {
                    Message::NewJob(job)  =>  {
                        println!("Worker {} : executing job", id);
                        job.call_box();
                    },
                    Message::Terminate => {
                        println!("Worker {} : terminating", id);
                        break;
                    },
                }

            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

pub trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)();
    }
}

pub type Job = Box<FnBox + Send + 'static>;
