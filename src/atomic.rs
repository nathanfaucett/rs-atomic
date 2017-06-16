use alloc::boxed::Box;

use core::sync::atomic::AtomicPtr;

pub use core::sync::atomic::Ordering;


pub struct Atomic<T> {
    ptr: AtomicPtr<T>,
}

unsafe impl<T> Send for Atomic<T> {}
unsafe impl<T> Sync for Atomic<T> {}

impl<T: Default> Default for Atomic<T> {

    #[inline(always)]
    fn default() -> Atomic<T> {
        Atomic::new(T::default())
    }
}

impl<T> Atomic<T> {

    #[inline(always)]
    pub fn new(value: T) -> Atomic<T> {
        Atomic {
            ptr: AtomicPtr::new(Box::into_raw(Box::new(value))),
        }
    }

    #[inline(always)]
    fn as_ptr(&self, order: Ordering) -> *mut T {
        self.ptr.load(order)
    }
    #[inline(always)]
    pub fn as_ref(&self, order: Ordering) -> &T {
        unsafe {
            &*self.as_ptr(order)
        }
    }
    #[inline(always)]
    pub fn as_mut(&mut self, order: Ordering) -> &mut T {
        unsafe {
            &mut *self.as_ptr(order)
        }
    }

    #[inline(always)]
    pub fn swap(&self, value: T, order: Ordering) -> T {
        unsafe {
            let ptr = Box::into_raw(Box::new(value));
            let old_ptr = self.ptr.swap(ptr, order);
            *Box::from_raw(old_ptr)
        }
    }
    #[inline(always)]
    pub fn store(&self, value: T, order: Ordering)  {
        self.swap(value, order);
    }
}

#[cfg(test)]
mod test {
    use super::*;


    struct Foo {
        bar: usize,
    }

    impl Foo {
        fn new(bar: usize) -> Self {
            Foo {
                bar: bar,
            }
        }
    }


    #[test]
    fn as_ref_test() {
        let value = Atomic::new(Foo::new(0));
        assert_eq!(value.as_ref(Ordering::Relaxed).bar, 0);
    }
    #[test]
    fn as_mut_test() {
        let mut value = Atomic::new(Foo::new(0));
        value.as_mut(Ordering::Relaxed).bar = 1;
        assert_eq!(value.as_ref(Ordering::Relaxed).bar, 1);
    }
    #[test]
    fn swap_test() {
        let value = Atomic::new(Foo::new(0));
        let old_value = value.swap(Foo::new(1), Ordering::Relaxed);
        assert_eq!(old_value.bar, 0);
        assert_eq!(value.as_ref(Ordering::Relaxed).bar, 1);
    }

    #[test]
    fn drop_test() {
        static mut DROPPED: bool = false;

        struct DroppableFoo {
            bar: usize,
        }

        impl DroppableFoo {
            fn new(bar: usize) -> Self {
                DroppableFoo {
                    bar: bar,
                }
            }
        }

        impl Drop for DroppableFoo {
            fn drop(&mut self) {
                unsafe {
                    DROPPED = true;
                }
            }
        }

        {
            assert_eq!(unsafe {DROPPED}, false);

            let value = Atomic::new(DroppableFoo::new(0));
            let old_value = value.swap(DroppableFoo::new(1), Ordering::Relaxed);

            assert_eq!(unsafe {DROPPED}, false);

            assert_eq!(old_value.bar, 0);
            assert_eq!(value.as_ref(Ordering::Relaxed).bar, 1);
        }

        assert_eq!(unsafe {DROPPED}, true);
    }

    #[test]
    fn threads() {
        use std::time;
        use std::thread;
        use std::vec::Vec;
        use alloc::arc::Arc;
        use core::sync::atomic::AtomicUsize;

        use prng::{AtomicPrng, MAX};


        static SIZE: usize = 32;

        let value = Arc::new(Atomic::new(Foo::new(0)));
        let count = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::new();
        let rand = Arc::new(AtomicPrng::new());

        for _ in 0..SIZE {
            let count = count.clone();
            let value = value.clone();
            let rand = rand.clone();

            handles.push(thread::spawn(move || {
                let ms = time::Duration::from_millis(((rand.next() as f64 / MAX as f64) * 128_f64) as u64);
                thread::sleep(ms);

                count.fetch_add(1, Ordering::Relaxed);

                let ms = time::Duration::from_millis(((rand.next() as f64 / MAX as f64) * 128_f64) as u64);
                thread::sleep(ms);

                value.swap(Foo::new(count.load(Ordering::Relaxed)), Ordering::Relaxed);
            }));
        }

        for handle in handles {
            let _ = handle.join();
        }

        assert_eq!((*value).as_ref(Ordering::Relaxed).bar, SIZE);
    }
}
