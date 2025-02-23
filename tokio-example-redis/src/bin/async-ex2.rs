use std::rc::Rc;

use tokio::task::yield_now;

async fn say_world() {
    println!("word");
}

//https://tokio.rs/tokio/tutorial/spawning
pub async fn wrong_with_send() {
    tokio::spawn(async {
        let rc = Rc::new("hello");

        drop(rc); //добавлено для компиляции
                  // `rc` is used after `.await`. It must be persisted to
                  // the task's state.
        yield_now().await;

        //println!("{}", rc); это не скомпилируется
    });
}

#[tokio::main]
async fn main() {
    // Calling `say_world()` does not execute the body of `say_world()`.
    say_world().await;
    println!("in the middle");
    say_world().await;
}
