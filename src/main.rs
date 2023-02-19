use std::sync::{mpsc::sync_channel, Mutex, mpsc::SyncSender, mpsc::Receiver};
use async_std::task::block_on;
use futures::FutureExt;
use futures::future::BoxFuture;
use std::thread;

struct Task<T> {
    fut: Mutex<BoxFuture<'static, T>>,
}

impl<T> Task<T> {
    pub fn new(f: BoxFuture<'static, T>) -> Self {
        Task {
            fut: Mutex::new(f),
        }
    }
}

async fn say_hello() -> String {
    println!("hello");
    String::from("hello")
}

async fn say_world() -> String {
    println!("world");
    String::from("world")
}

async fn say_bye() -> String {
    println!("bye");
    String::from("bye")
}

async fn excutor(queue: Receiver<Task<String>>) {
    let mut count = 0;
    loop {
        match queue.recv() {
           Ok(task) => {
                let r = task.fut.lock().unwrap().as_mut().await;
                println!("await return: {}", r);
                count += 1;
                if count == 3 {
                    return ;
                }
           } 
           Err(_) =>  {

           }
        }
    }
}

fn main() {
    // 创建消息队列
    let (sender, queue) = sync_channel::<Task<String>>(10);

    // 创建三个即将被异步调用的函数，其返回 future
    let hello_fut = say_hello();
    let world_fut = say_world();
    let bye_fut = say_bye();

    // 创建三个Task，存放之前三个函数的future
    let t1 = Task::new(hello_fut.boxed());
    let t2 = Task::new(world_fut.boxed());
    let t3 = Task::new(bye_fut.boxed());

    // 以下创建三个生产者线程，用以模拟多个IO生产的数据操作
    // 第一个生产者线程
    let sender1 = sender.clone();
    thread::spawn(move || {
        sender1.send(t1).unwrap();
    }); 

    // 第二个生产者线程
    let sender2 = sender.clone();
    thread::spawn(move || {
        sender2.send(t2).unwrap();
    }); 

    // 第三个生产者线程
    let sender3 = sender.clone();
    thread::spawn(move || {
        sender3.send(t3).unwrap();
    }); 

    // 执行消费者工作，从队列中取出Task，触发await
    block_on(excutor(queue));
}
