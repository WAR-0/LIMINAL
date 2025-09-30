use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::sync::mpsc;
use tokio::task::JoinSet;

type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

#[derive(Clone)]
pub struct MaintenanceExecutor {
    sender: mpsc::UnboundedSender<BoxedFuture>,
    handle: Handle,
}

impl MaintenanceExecutor {
    pub fn new(_worker_count: usize) -> Self {
        let handle = match Handle::try_current() {
            Ok(h) => h,
            Err(_) => {
                let runtime =
                    tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
                runtime.handle().clone()
            }
        };

        let (sender, mut receiver) = mpsc::unbounded_channel::<BoxedFuture>();

        handle.spawn(async move {
            let mut tasks = JoinSet::new();

            loop {
                tokio::select! {
                    Some(task) = receiver.recv() => {
                        tasks.spawn(async move {
                            task.await;
                        });
                    }
                    Some(result) = tasks.join_next() => {
                        if let Err(e) = result {
                            eprintln!("Task failed: {:?}", e);
                        }
                    }
                    else => break,
                }

                while tasks.len() > 100 {
                    if let Some(result) = tasks.join_next().await {
                        if let Err(e) = result {
                            eprintln!("Task failed: {:?}", e);
                        }
                    }
                }
            }
        });

        Self { sender, handle }
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let _ = self.sender.send(Box::pin(future));
    }

    pub fn inner(&self) -> Arc<()> {
        Arc::new(())
    }

    pub fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            handle: self.handle.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_executor_spawns_tasks() {
        let executor = MaintenanceExecutor::new(4);
        let counter = Arc::new(AtomicUsize::new(0));

        for _ in 0..10 {
            let counter = counter.clone();
            executor.spawn(async move {
                counter.fetch_add(1, Ordering::SeqCst);
            });
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }
}
