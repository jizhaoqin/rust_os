#![allow(dead_code)]

use crate::{print, println};
use conquer_once::spin::OnceCell;
use core::pin::Pin;
use core::task::{Context, Poll};
use crossbeam_queue::ArrayQueue;
use futures_util::stream::{Stream, StreamExt};
use futures_util::task::AtomicWaker;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

/// 由keyboard_interrupt_handler()调用
///
/// - 这里是中断上下文, 所以must not block or allocate
/// - 这里把scancode加入异步流[`Stream`]
/// - 最后唤醒异步执行器尝试进行处理
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        //
        if queue.push(scancode).is_err() {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            // 中断处理的最后执行唤醒操作
            // 具体逻辑是从virtual table调用了我们通过impl Wake for TaskWaker传入的函数
            // 效果是将print_key_presses()的id重新加入所属执行器的task_queue
            // 之后执行器将poll print_key_presses(), 进入函数体打印字符
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

/// 这里是async和正常函数的分离点
///
/// - 再往上调用这个函数的是非async但是阻塞的执行器, 这个执行器在空闲时应该yield, 这里是因为没有其他任务所以一直占用CPU (main.rs)
/// - 需要注意的是, 这个async函数实际上永远只会返回pending, 因为我们很容易看到while循环没有break语句, 也就是一个无限循环,
///   但只有键盘中断发生时这个函数才会被poll
/// - 每次按一个键这个函数被poll两次, 这是因为按下和松开是两个事件
/// - 而且如果多个按键同时按下, 也只会被poll两次, 当作一次中断, 似乎有批量处理的功能
/// - poll这个future的时候传入了context
/// - 执行器初始化的时候加入此future时, 会poll一次, 因为这不是由中断引起的
/// - 在这首次poll的时候, 会由执行器新建并传入包含执行器信息的waker, 之后这个future就一直在pending
pub async fn print_key_presses() {
    let mut scan_codes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    // println!("poll print_key_presses()");

    // .await这里相当于poll了scan_codes.next()这个future, 也传入了context信息
    // 这里逻辑上是阻塞的, 因为我们并没有实现线程和抢占式多任务
    while let Some(scancode) = scan_codes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}

/// 字段 _private 的目的是防止從模塊外部構造結構(可以去除)
#[derive(Default)]
struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

/// Future trait 只是對單個異步值進行抽象，並且期望 poll 方法在返回 Poll::Ready 後不再被調用
/// 然而，我們的掃描碼隊列包含多個異步值，所以保持對它的輪詢是可以的
/// `Stream` trait适用于產生多個異步值的類型
/// - Future poll()      -> Poll<Output>
/// - Stream poll_next() -> Poll<Option<Item>>
impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialized");

        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        // 这里注册的`Waker`实际上来自于TaskWaker::new_waker(task_id, task_queue.clone())
        WAKER.register(cx.waker());
        match queue.pop() {
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}
