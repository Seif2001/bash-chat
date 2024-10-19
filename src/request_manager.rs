use std::sync::mpsc::Receiver;
use std::{thread, time};

pub fn test_thread(rx: Receiver<String>) -> thread::JoinHandle<()> {
    let thread1 = thread::spawn(move || {
        for received in rx {
            println!("message process thread: {}", received);
            //processing would be done here but theres no image processor yet
        }
    });
    thread1
}

pub fn dummy_thread() -> thread::JoinHandle<()> {
    //i put this to simulate that multiple threads can run at once it literally just prints numbers
    let thread1 = thread::spawn(move || {
        for i in 1..1000000 {
            println!("Dummy Thread: {i}");
            thread::sleep(time::Duration::from_millis(600)); //sleep so we dont clutter the terminal
        }
    });
    thread1
}
