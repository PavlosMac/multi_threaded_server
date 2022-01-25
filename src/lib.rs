use std::thread;
use std::sync::mpsc;
use std::sync::Mutex;
use std::sync::Arc;

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
            let message = receiver.lock().unwrap().recv().unwrap();

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

        Worker { id, thread: Some(thread) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_has_id() {
        let (_, receiver) = mpsc::channel();
        let returned_worker = Worker::new( 2,Arc::new(Mutex::new(Message::NewJob(2) )));

        assert_eq!(returned_worker.id, 2);
    }

    #[test]
    #[should_panic]
    fn worker_id_is_too_low() {
        Worker::new( 0,Arc::new(Mutex::new(receiver )));
    }
}
