use futures_channel::oneshot::{Receiver, channel};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use std::future::Future;
use std::task::{Context, Poll};
use std::pin::Pin;

/// Runs a Rust `Future` on the current thread.
///
/// The `future` must be `'static` because it will be scheduled
/// to run in the background and cannot contain any stack references.
///
/// The `future` will always be run on the next microtask tick even if it
/// immediately returns `Poll::Ready`.
///
/// # Panics
///
/// Note that in wasm panics are currently translated to aborts, but "abort" in
/// this case means that a JavaScript exception is thrown. The wasm module is
/// still usable (likely erroneously) after Rust panics.
///
/// If the `future` provided panics then the returned `Promise` **will not
/// resolve**. Instead it will be a leaked promise. This is an unfortunate
/// limitation of wasm currently that's hoped to be fixed one day!
#[inline]
pub fn spawn_local<F, T>(future: F) -> JoinHandle<T>
where
    F: Future<Output = T> + 'static,
    T: 'static,
{
    let (sender, receiver) = channel();
    let fut = async move {
        let t = future.await;
        let _ = sender.send(t);
    };

    wasm_bindgen_futures::spawn_local(fut);
    JoinHandle { receiver }
}

/// Task priority.
#[derive(Debug)]
pub enum Priority {
    /// Spawns a task on the microqueue.
    High,
    /// Spawns a task when the browser has idle time.
    Low,
}

/// A handle that awaits the result of a [`spawn`]ed future.
///
/// [`spawn`]: fn.spawn.html
#[derive(Debug)]
pub struct JoinHandle<T> {
    pub(crate) receiver: Receiver<T>,
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.receiver).poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(t)) => Poll::Ready(t),
            Poll::Ready(Err(_)) => unreachable!(),
        }
    }
}

/// Spawn a task that runs when the event loop is idle.
#[inline]
pub fn spawn_idle<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + 'static,
    T: 'static + Send,
{
    let (sender, receiver) = channel();
    let f2 = Closure::once(Box::new(move || {
        let t = f();
        let _ = sender.send(t);
    }) as Box<dyn FnOnce()>);

    let _ = crate::window().request_idle_callback(f2.as_ref().unchecked_ref());
    JoinHandle { receiver }
}
