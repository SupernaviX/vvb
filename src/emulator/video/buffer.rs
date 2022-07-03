use super::FRAME_SIZE;
use std::ops::Index;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::TryLockError::WouldBlock;
use std::sync::{Arc, Mutex};

fn collect_to_array<T, I, const N: usize>(iterator: I) -> [T; N]
where
    I: Iterator<Item = T>,
    T: Default + Copy,
{
    let mut result = [Default::default(); N];
    for (it, elem) in result.iter_mut().zip(iterator) {
        *it = elem
    }
    result
}

type BufferData = Box<[u8; FRAME_SIZE]>;
struct Buffer {
    gen: AtomicUsize,
    data: Mutex<BufferData>,
}
impl Buffer {
    fn new(gen: usize) -> Self {
        let data = {
            let allocated = vec![0u8; FRAME_SIZE].into_boxed_slice();
            let pointer = Box::into_raw(allocated) as *mut [u8; FRAME_SIZE];
            Mutex::new(unsafe { Box::from_raw(pointer) })
        };
        Self {
            gen: AtomicUsize::new(gen),
            data,
        }
    }
}

struct Buffers([Buffer; 3]);
impl Buffers {
    fn generations(&self) -> impl Iterator<Item = usize> + '_ {
        self.0.iter().map(|b| b.gen.load(Ordering::Acquire))
    }
    fn newest(&self) -> (&Buffer, usize) {
        self.0
            .iter()
            .map(|b| (b, b.gen.load(Ordering::Acquire)))
            .max_by_key(|(_, gen)| *gen)
            .unwrap()
    }
}

impl Default for Buffers {
    fn default() -> Self {
        Self([Buffer::new(0), Buffer::new(1), Buffer::new(2)])
    }
}
impl Index<usize> for Buffers {
    type Output = Buffer;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

#[derive(Default)]
pub struct SharedBuffer {
    buffers: Arc<Buffers>,
}

impl SharedBuffer {
    #[test]
    pub fn read<F>(&self, consumer: F)
    where
        F: FnOnce(&[u8; FRAME_SIZE]),
    {
        let (buffer, _) = self.buffers.newest();
        let guard = buffer.data.lock().expect("Buffer lock was poisoned");

        consumer(&guard);
    }

    pub fn write<F>(&self, producer: F)
    where
        F: FnOnce(&mut [u8; FRAME_SIZE]),
    {
        let mut sorted_indices_and_gens: [_; 3] =
            collect_to_array(self.buffers.generations().enumerate());
        sorted_indices_and_gens.sort_by_key(|(_, gen)| *gen);
        let min_index = sorted_indices_and_gens[0].0;
        let mid_index = sorted_indices_and_gens[1].0;
        let new_generation = sorted_indices_and_gens[2].1 + 1;

        let (buffer, mut guard) = {
            let min_buffer = &self.buffers[min_index];
            match min_buffer.data.try_lock() {
                Ok(guard) => (min_buffer, guard),
                Err(WouldBlock) => {
                    let mid_buffer = &self.buffers[mid_index];
                    (
                        mid_buffer,
                        mid_buffer.data.lock().expect("Buffer lock was poisoned!"),
                    )
                }
                Err(_) => panic!("Buffer lock was poisoned!"),
            }
        };

        producer(&mut guard);
        buffer.gen.store(new_generation, Ordering::Release);
    }

    pub fn consumer(&self) -> SharedBufferConsumer {
        SharedBufferConsumer {
            buffers: Arc::clone(&self.buffers),
            last_generation: 0,
        }
    }
}

pub struct SharedBufferConsumer {
    buffers: Arc<Buffers>,
    last_generation: usize,
}
impl SharedBufferConsumer {
    pub fn try_read<F>(&mut self, consumer: F)
    where
        F: FnOnce(&[u8; FRAME_SIZE]),
    {
        let (buffer, generation) = self.buffers.newest();
        if self.last_generation < generation {
            let guard = buffer.data.lock().expect("Buffer lock was poisoned!");
            consumer(&guard);
            self.last_generation = buffer.gen.load(Ordering::SeqCst);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::emulator::video::buffer::SharedBuffer;
    use std::sync::mpsc;
    use std::thread;
    use std::thread::JoinHandle;
    use std::time::Duration;

    const DEFAULT_TIMEOUT: Duration = Duration::from_secs(2);

    fn panic_after<T, F>(d: Duration, f: F) -> T
    where
        T: Send + 'static,
        F: FnOnce() -> T,
        F: Send + 'static,
    {
        let (done_tx, done_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            let val = f();
            done_tx.send(()).expect("Unable to send completion signal");
            val
        });

        match done_rx.recv_timeout(d) {
            Ok(_) => handle.join().expect("Thread panicked"),
            Err(_) => panic!("Thread took too long"),
        }
    }

    struct ThreadWrapper<T> {
        tx: mpsc::Sender<()>,
        handle: JoinHandle<T>,
    }
    impl<T> ThreadWrapper<T> {
        fn start(&self) {
            self.tx.send(()).unwrap();
        }
        fn join(self) -> T {
            self.handle.join().unwrap()
        }
    }
    fn in_another_thread<F, T>(f: F) -> ThreadWrapper<T>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            rx.recv().unwrap();
            f()
        });
        ThreadWrapper { tx, handle }
    }

    #[test]
    fn buffer_can_read_own_writes() {
        panic_after(DEFAULT_TIMEOUT, || {
            let buffer = SharedBuffer::default();
            buffer.write(|data| {
                data[1337] = 42;
            });

            let mut read = 0;
            buffer.read(|data| {
                read = data[1337];
            });
            assert_eq!(read, 42);
        });
    }

    #[test]
    fn consumer_can_read_buffer_writes_from_thread() {
        panic_after(DEFAULT_TIMEOUT, || {
            let buffer = SharedBuffer::default();
            let mut consumer = buffer.consumer();

            let thread = in_another_thread(move || {
                let mut read = 0;
                consumer.try_read(|data| {
                    read = data[1337];
                });
                read
            });

            buffer.write(|data| {
                data[1337] = 128;
            });

            thread.start();
            assert_eq!(thread.join(), 128);
        });
    }

    #[test]
    fn consumer_does_not_block_on_producer() {
        panic_after(DEFAULT_TIMEOUT, || {
            let buffer = SharedBuffer::default();
            let mut consumer = buffer.consumer();
            let thread = in_another_thread(move || {
                let mut read = 0;
                consumer.try_read(|data| {
                    read = data[1337];
                });
                read
            });

            buffer.write(|data| {
                data[1337] = 64;
            });

            let mut read = 0;
            buffer.write(|data| {
                thread.start();

                data[1337] = 128;

                read = thread.join();
            });
            assert_eq!(read, 64);
        });
    }

    #[test]
    fn producer_does_not_block_on_consumer() {
        panic_after(DEFAULT_TIMEOUT, || {
            let buffer = SharedBuffer::default();
            let mut consumer = buffer.consumer();

            buffer.write(|data| {
                data[1337] = 1;
            });

            let thread = in_another_thread(move || {
                buffer.write(|data| {
                    data[1337] = 2;
                });
                buffer.write(|data| {
                    data[1337] = 3;
                });
                buffer.write(|data| {
                    data[1337] = 4;
                })
            });

            let mut read = 0;
            consumer.try_read(|data| {
                thread.start();
                read = data[1337];
                thread.join();
            });
            assert_eq!(read, 1);
        });
    }

    #[test]
    fn consumer_does_not_do_anything_when_no_new_data() {
        panic_after(DEFAULT_TIMEOUT, || {
            let buffer = SharedBuffer::default();
            let mut consumer = buffer.consumer();

            buffer.write(|data| {
                data[1337] = 42;
            });

            let mut reads = 0;
            consumer.try_read(|_data| {
                reads += 1;
            });
            consumer.try_read(|_data| {
                reads += 1;
            });

            assert_eq!(reads, 1);
        });
    }
}
