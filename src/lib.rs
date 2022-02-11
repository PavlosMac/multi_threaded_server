use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

enum Message {
    NewJob(Job),
    Terminate,
}
// A thread pool is a group of spawned threads that are waiting and ready to handle a task.
// When the program receives a new task, it assigns one of the threads in the pool to the task,
// and that thread will process the task.
impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel(); // creates a new async channel

        let receiver = Arc::new(Mutex::new(receiver)); // Arc used to share memory

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        for worker in &mut self.workers {
            println!("Shutting down worker {} ", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().expect("previous thread panicked....").recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a new job; executing", id);
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} was told to stop executing", id);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::JoinHandle;
    use crate::Message::NewJob;
    use std::*;
    use std::io::Write;

    #[test]
    fn test_thread_pool_new() {
        let pool = ThreadPool::new(10);

        assert_eq!(pool.workers.len(), 10);
    }

    #[test]
    fn test_thread_execute() {
        let pool = ThreadPool::new(1);
        let mut stdout = Vec::new();

        let x = {
            write_to_stdout(&mut stdout, "...processing task 1");
            write_to_stdout(&mut stdout, "...processing task 2");
        };

        let consume_fn = move || x;
        pool.execute( consume_fn );

        assert_eq!(stdout, b"...processing task 1\n...processing task 2\n");
    }

    #[test]
    fn test_worker_new() {
        let pool = ThreadPool::new(1);
        let mut stdout = Vec::new();
        let (sender, receiver) = mpsc::channel(); // creates a new async channel
    
        let receiver = Arc::new(Mutex::new(receiver)); // Arc used to share memory
    
        Worker::new(5, Arc::clone(&receiver));
    
        let x = {
            write_to_stdout(&mut stdout, "...processing task 1");
            write_to_stdout(&mut stdout, "...processing task 2");
        };
    
        let consume_fn = move || x;
        pool.execute( consume_fn );
    
        assert_eq!(stdout, b"...processing task 1\n...processing task 2\n");
    }

    fn write_to_stdout(stdout: &mut dyn io::Write, thing: &str) {
        writeln!(stdout, "{}", thing);
    }
}
