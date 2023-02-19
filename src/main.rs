use std::{sync::{mpsc::sync_channel, 
                Mutex, 
                mpsc::SyncSender, 
                mpsc::Receiver, Arc}, task::Context};
use async_std::task::block_on;
use futures::{
    Future,
    future::{
        BoxFuture,
    },
    task::{
        ArcWake,
        waker_ref,
        Waker,
        Poll,
    }, FutureExt
};
use std::thread;

struct Task<T> {
    fut: Mutex<Option<BoxFuture<'static, T>>>,
    sender: SyncSender<Arc<Task<T>>>,
}

impl<T> Task<T> {
    pub fn new(sender: SyncSender<Arc<Task<T>>>, f: BoxFuture<'static, T>) -> Self {
        Task {
            fut: Mutex::new(Some(f)),
            sender,
        }
    }
}

impl<T> ArcWake for Task<T> {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.sender.send(arc_self.clone()).expect("failed to send task");
    }
}

struct SayHelloInPending {
    shared_data: Arc<Mutex<SharedData>>,
}

struct SharedData {
    completed: bool,
    waker: Option<Waker>,
}

impl SayHelloInPending {
   fn new(millis: u64) -> Self  {
        let task = SayHelloInPending { 
            shared_data: Arc::new(Mutex::new(SharedData { 
                completed: false, 
                waker: None 
            })) 
        };
        let shared_data_clone = Arc::clone(&task.shared_data);
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(millis));
            let mut data = shared_data_clone.lock().unwrap();
            data.completed = true;
            if let Some(w) = data.waker.take() {
                println!("wake??");
                w.wake();
            } else {
                println!("no wake??");
            }
        });
        task
   }
}

impl Future for SayHelloInPending {
    type Output = String;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        println!("poll!!??");
        let shared_data = self.shared_data.lock();
        println!("lock poll!!??");
        if let Ok(mut data) = shared_data {
            if data.completed {
                return Poll::Ready("completed".to_string());
            } else {
                data.waker = Some(cx.waker().clone());
            }         
        }
        Poll::Pending
    }
}

async fn excutor(queue: Receiver<Arc<Task<String>>>) {
    loop {
        match queue.recv() {
           Ok(task) => {
                let waker = waker_ref(&task);
                let contex = &mut Context::from_waker(&*waker);
                let mut fut = task.fut.lock().unwrap();
                if let Some(mut f) = fut.take() {
                    let result = f.as_mut().poll(contex);
                    if  result.is_pending() {
                        *fut = Some(f);
                    } else if let Poll::Ready(s) = result {
                        println!("finish and receive: {}", s);
                        break;
                    }
                }
           } 
           Err(_) =>  {

           }
        }
    }
}

fn main() {
    // 创建消息队列
    let (sender, queue) = sync_channel::<Arc<Task<String>>>(10);

    // 创建三个即将被异步调用的函数，其返回 future
    let mut task = Task::<String>::new(sender.clone(), 
                                                     SayHelloInPending::new(2 * 1000).boxed());

    sender.send(Arc::new(task)).unwrap();
    
    // 执行消费者工作，从队列中取出Task，触发await
    block_on(excutor(queue));
}
