use std::sync::Arc;
use tokio::sync::Mutex;

pub type RtConsumer<T> = rtrb::Consumer<T>;
pub type RtProducer<T> = rtrb::Producer<T>;
pub type PushError<T> = rtrb::PushError<T>;
pub type PopError = rtrb::PopError;

pub struct RtSharedProducer<T> {
    inner: Arc<Mutex<rtrb::Producer<T>>>,
}

impl<T> Clone for RtSharedProducer<T> {
    fn clone(&self) -> Self {
        Self { inner: Arc::clone(&self.inner) }
    }
}

impl<T> RtSharedProducer<T> {
    pub async fn push(&self, value: T) -> Result<(), T> {
        let mut producer = self.inner.lock().await;
        producer.push(value).map_err(|err| match err {
            rtrb::PushError::Full(value) => value,
        })
    }

    pub fn blocking_push(&self, value: T) -> Result<(), T> {
        let mut producer = self.inner.blocking_lock();
        producer.push(value).map_err(|err| match err {
            rtrb::PushError::Full(value) => value,
        })
    }

    pub fn inner(&self) -> &Arc<Mutex<rtrb::Producer<T>>> {
        &self.inner
    }
}

pub struct RtSharedConsumer<T> {
    inner: Arc<Mutex<rtrb::Consumer<T>>>,
}

impl<T> Clone for RtSharedConsumer<T> {
    fn clone(&self) -> Self {
        Self { inner: Arc::clone(&self.inner) }
    }
}

impl<T> RtSharedConsumer<T> {
    pub async fn pop(&self) -> Option<T> {
        let mut consumer = self.inner.lock().await;
        consumer.pop().ok()
    }

    pub fn blocking_pop(&self) -> Option<T> {
        let mut consumer = self.inner.blocking_lock();
        consumer.pop().ok()
    }

    pub fn inner(&self) -> &Arc<Mutex<rtrb::Consumer<T>>> {
        &self.inner
    }
}

pub struct NotifiedRtSharedProducer<T> {
    producer: RtSharedProducer<T>,
    notify: Arc<tokio::sync::Notify>,
}

impl<T> Clone for NotifiedRtSharedProducer<T> {
    fn clone(&self) -> Self {
        Self { producer: self.producer.clone(), notify: Arc::clone(&self.notify) }
    }
}

impl<T> NotifiedRtSharedProducer<T> {
    pub fn blocking_push(&self, value: T) -> Result<(), T> {
        match self.producer.blocking_push(value) {
            Ok(()) => {
                self.notify.notify_one();
                Ok(())
            }
            Err(value) => Err(value),
        }
    }

    pub async fn push(&self, value: T) -> Result<(), T> {
        match self.producer.push(value).await {
            Ok(()) => {
                self.notify.notify_one();
                Ok(())
            }
            Err(value) => Err(value),
        }
    }

    pub fn clone_notify(&self) -> Arc<tokio::sync::Notify> {
        Arc::clone(&self.notify)
    }
}

pub struct NotifiedRtConsumer<T> {
    consumer: RtConsumer<T>,
    notify: Arc<tokio::sync::Notify>,
}

impl<T> NotifiedRtConsumer<T> {
    pub async fn recv(&mut self) -> T {
        loop {
            if let Ok(value) = self.consumer.pop() {
                return value;
            }

            self.notify.notified().await;
        }
    }

    pub fn try_recv(&mut self) -> Result<T, PopError> {
        self.consumer.pop()
    }

    pub fn unwrap(self) -> (RtConsumer<T>, Arc<tokio::sync::Notify>) {
        (self.consumer, self.notify)
    }
}

pub fn create_mpmc<T>(capacity: usize) -> (RtSharedProducer<T>, RtSharedConsumer<T>) {
    let (producer, consumer) = rtrb::RingBuffer::new(capacity);
    let shared_producer = RtSharedProducer { inner: Arc::new(Mutex::new(producer)) };
    let shared_consumer = RtSharedConsumer { inner: Arc::new(Mutex::new(consumer)) };
    (shared_producer, shared_consumer)
}

pub fn create_mpsc<T>(capacity: usize) -> (RtSharedProducer<T>, RtConsumer<T>) {
    let (producer, consumer) = rtrb::RingBuffer::new(capacity);
    let shared_producer = RtSharedProducer { inner: Arc::new(Mutex::new(producer)) };
    (shared_producer, consumer)
}

pub fn create_spmc<T>(capacity: usize) -> (RtProducer<T>, RtSharedConsumer<T>) {
    let (producer, consumer) = rtrb::RingBuffer::new(capacity);
    let shared_consumer = RtSharedConsumer { inner: Arc::new(Mutex::new(consumer)) };
    (producer, shared_consumer)
}

pub fn create_spsc<T>(capacity: usize) -> (RtProducer<T>, RtConsumer<T>) {
    rtrb::RingBuffer::new(capacity)
}

pub fn create_notified_mpsc<T>(
    capacity: usize,
) -> (NotifiedRtSharedProducer<T>, NotifiedRtConsumer<T>) {
    let (producer, consumer) = create_mpsc(capacity);
    let notify = Arc::new(tokio::sync::Notify::new());
    let notified_producer = NotifiedRtSharedProducer { producer, notify: Arc::clone(&notify) };
    let notified_consumer = NotifiedRtConsumer { consumer, notify };
    (notified_producer, notified_consumer)
}
