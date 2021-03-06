use std::sync::{Arc, Mutex};
use wasm_bindgen_test::*;

use localghost::task::AnimationLoop;

#[wasm_bindgen_test]
async fn animation_loop() {
    let count = Arc::new(Mutex::new(0_u8));
    let mut animator = AnimationLoop::new(|count: Arc<Mutex<u8>>| async move {
        *count.lock().unwrap() += 1;
    });

    animator.render(count.clone()).await;
    assert_eq!(*count.lock().unwrap(), 1);

    animator.render(count.clone()).await;
    assert_eq!(*count.lock().unwrap(), 2);
}

#[wasm_bindgen_test]
async fn spawn_local() {
    let handle = localghost::task::spawn_local(async move { 12_u8 });
    assert_eq!(handle.await, 12);
}

// #[wasm_bindgen_test]
// async fn spawn_idle() {
//     let handle = localghost::task::spawn_idle(|| {
//         12_u8
//     });
//     assert_eq!(handle.await, 12);
// }
