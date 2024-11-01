use std::sync::mpsc;

pub mod request_manager;

/*

what this program does is allow a user to input a charaacter which is received by another thread to "process"
Simulataneously another thread just prints numbers

*/
fn main() {
    let (tx, rx) = mpsc::channel();
    let thread1 = request_manager::test_thread(rx);
    let thread2 = request_manager::dummy_thread();

    println!("calling threads finished"); // origianlly used to testing what exactly join does by returning after this

    let mut usr_input = String::new();
    std::io::stdin()
        .read_line(&mut usr_input)
        .expect("failed to read");
    //putting in while loop to simulate
    while usr_input.trim() != "c" {
        let _ = tx.send(usr_input.clone());
        usr_input.clear();
        std::io::stdin()
            .read_line(&mut usr_input)
            .expect("failed to read");
    }
    drop(tx);
    thread1
        .join()
        .expect("failed to join");
    thread2
        .join()
        .expect("failed to join");
}
