use crate::rt_channels;
use std::{
    mem,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering, fence},
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use tracing::error;

pub struct Retirer<T> {
    producer: rt_channels::RtProducer<T>,
    failed_retire_count: Arc<AtomicU64>,
}

pub struct RetirementThread {
    handle: Option<JoinHandle<()>>,
    failed_retire_count: Arc<AtomicU64>,
}

impl RetirementThread {
    pub fn spawn<T>(
        capacity: usize,
        idle_sleep: Duration,
    ) -> std::io::Result<(Retirer<T>, RetirementThread)>
    where
        T: Send + 'static,
    {
        let (producer, consumer) = rt_channels::create_spsc(capacity);
        let failed_retire_count = Arc::new(AtomicU64::new(0));

        let handle = thread::Builder::new()
            .name("retirement-thread".to_string())
            .spawn(move || retirement_loop(consumer, idle_sleep))?;

        let retirer = Retirer { producer, failed_retire_count: failed_retire_count.clone() };

        let retirement_thread = RetirementThread { handle: Some(handle), failed_retire_count };

        Ok((retirer, retirement_thread))
    }

    pub fn failed_retire_count(&self) -> u64 {
        self.failed_retire_count.load(Ordering::Relaxed)
    }

    pub fn join(mut self) -> thread::Result<()> {
        if let Some(handle) = self.handle.take() { handle.join() } else { Ok(()) }
    }

    pub fn join_after_dropping<T>(self, retirer: Retirer<T>) -> thread::Result<()> {
        drop(retirer);
        self.join()
    }
}

impl<T> Retirer<T> {
    pub fn try_retire(&mut self, object: T) -> Result<(), T> {
        if self.producer.is_abandoned() {
            // This is needed since Rust 1.74.0, see https://github.com/mgeier/rtrb/issues/114
            fence(Ordering::Acquire);

            self.failed_retire_count.fetch_add(1, Ordering::Relaxed);
            return Err(object);
        }

        match self.producer.push(object) {
            Ok(()) => Ok(()),

            Err(rt_channels::PushError::Full(object)) => {
                self.failed_retire_count.fetch_add(1, Ordering::Relaxed);
                Err(object)
            }
        }
    }

    pub fn retire_or_leak(&mut self, object: T) {
        if let Err(object) = self.try_retire(object) {
            mem::forget(object);
            error!("failed to retire object: retirement queue full; memory leaked");
        }
    }

    pub fn failed_retire_count(&self) -> u64 {
        self.failed_retire_count.load(Ordering::Relaxed)
    }

    pub fn retirement_thread_is_gone(&self) -> bool {
        let abandoned = self.producer.is_abandoned();

        if abandoned {
            // This is needed since Rust 1.74.0, see https://github.com/mgeier/rtrb/issues/114
            fence(Ordering::Acquire);
        }

        abandoned
    }
}

fn retirement_loop<T>(mut consumer: rt_channels::RtConsumer<T>, idle_sleep: Duration) {
    loop {
        let drained = drain(&mut consumer);

        if consumer.is_abandoned() {
            // This is needed since Rust 1.74.0, see https://github.com/mgeier/rtrb/issues/114
            fence(Ordering::Acquire);

            drain(&mut consumer);
            break;
        }

        if drained == 0 {
            thread::sleep(idle_sleep);
        }
    }
}

fn drain<T>(consumer: &mut rt_channels::RtConsumer<T>) -> usize {
    let mut count = 0;

    while let Ok(object) = consumer.pop() {
        drop(object);
        count += 1;
    }

    count
}
