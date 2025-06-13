use super::{Task, TaskId};
use crate::println;
use alloc::{collections::BTreeMap, sync::Arc};
use core::task::{Context, Poll, Waker};
use crossbeam_queue::ArrayQueue;

struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl alloc::task::Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    } // rust API, 定义pending的future如何唤醒执行器
}

impl Default for TaskWaker {
    fn default() -> Self {
        Self {
            task_id: TaskId(0),
            task_queue: Arc::new(ArrayQueue::new(1)),
        }
    }
}

impl TaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Self {
        TaskWaker {
            task_id,
            task_queue,
        }
    }

    fn new_waker(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(Self::new(task_id, task_queue)))
    }

    /// 具体的waker唤醒逻辑
    ///
    /// - 将task_id重新加入task_queue, 这样执行器在下一次轮询时会访问到此task
    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }
}

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    /// 生成任务加入队列中
    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID already in tasks");
        }
        self.task_queue.push(task_id).expect("queue full");
    }

    /// 轮询队列中的任务
    ///
    /// - 轮询task_queue中所有的task一遍, 这个队列只储存`id`
    /// - 因此pop掉之后不应先task和waker的存储
    /// - 但这意味着一次轮询如果某个task在pending, 那么下次轮询就没有它了
    /// - 也就是说所有任务执行器只会主动轮询一次
    fn run_ready_tasks(&mut self) {
        // pattern match
        let Executor {
            tasks,
            task_queue,
            waker_cache,
        } = self;

        // task_queue为空时推出循环, 所以这里并非无限循环
        while let Some(task_id) = task_queue.pop() {
            // println!("{:?} {:?}", task_queue.is_empty(), task_id);

            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // 任务不存在
            };

            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new_waker(task_id, task_queue.clone()));

            // 创建context准备poll一个task, 这里的context最关键的是包含task_queue的Arc引用
            // 也就是隐含了executor的信息, 因为task_queue是由执行器创建的, 也是执行器轮询的依据
            let mut context = Context::from_waker(waker);
            // 这里poll一个task也就是poll一个future, 并传入context包含执行器信息, 方便在pending的时候唤醒
            match task.poll(&mut context) {
                // 如果人物完成, 就把执行器中的所有相关信息删除, 包括tasks, task_queue, waker_cache
                Poll::Ready(()) => {
                    println!("remove once");

                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                // 这里并不会在把还在pending的task id加回队列, 所以下次轮询不会再询问这个任务
                Poll::Pending => {
                    println!("poll only once");
                }
            }
        }
    }

    /// 执行器运行暴露的接口
    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    /// 每次run_ready_tasks()只会轮询一遍
    ///
    /// - 之后如果所有任务都在pending, 那么大部分时间都会在执行这个idle函数
    /// - 如果有其他的任务要执行, 比如说在其他线程, 可以在这里yield
    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts::{self, enable_and_hlt};

        interrupts::disable();
        if self.task_queue.is_empty() {
            enable_and_hlt();
        } else {
            interrupts::enable();
        }
    }
}
