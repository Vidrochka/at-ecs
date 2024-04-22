use std::{alloc::Layout, mem::{align_of, size_of}, ptr::NonNull};

#[derive(Debug)]
pub struct RawAlloc<T> {
    ptr: NonNull<T>,
    cap: usize,
}

impl<T> RawAlloc<T> {
    fn new(min_capacity: usize) -> Self {
        if size_of::<T>() == 0 {
            Self {
                ptr: NonNull::dangling(),
                cap: !0,
            }
        } else if min_capacity == 0 {
            Self {
                ptr: NonNull::dangling(),
                cap: 0,
            }
        } else {
            let layout =
                Layout::from_size_align(size_of::<T>() * min_capacity, align_of::<T>()).unwrap();
            Self {
                ptr: NonNull::new(unsafe { std::alloc::alloc(layout) as *mut _ }).unwrap(),
                cap: min_capacity,
            }
        }
    }

    fn layout(&self) -> Layout {
        Layout::from_size_align(size_of::<T>() * self.cap, align_of::<T>()).unwrap()
    }

    fn grow(&mut self, new_capacity: usize) {
        debug_assert!(self.cap < new_capacity);

        unsafe {
            let dst_ptr = if self.cap == 0 {
                let layout =
                    Layout::from_size_align(size_of::<T>() * new_capacity, align_of::<T>())
                        .unwrap();
                std::alloc::alloc(layout) as *mut T
            } else {
                std::alloc::realloc(
                    self.ptr.as_ptr() as *mut u8,
                    self.layout(),
                    size_of::<T>() * new_capacity,
                ) as *mut T
            };
            if let Some(new_ptr) = NonNull::new(dst_ptr) {
                self.ptr = new_ptr;
                self.cap = new_capacity;
            } else {
                std::alloc::handle_alloc_error(Layout::from_size_align_unchecked(
                    size_of::<T>() * new_capacity,
                    align_of::<T>(),
                ));
            }
        }
    }
}

impl<T> Drop for RawAlloc<T> {
    fn drop(&mut self) {
        if size_of::<T>() != 0 && self.cap > 0 {
            unsafe {
                let layout =
                    Layout::from_size_align_unchecked(size_of::<T>() * self.cap, align_of::<T>());
                std::alloc::dealloc(self.ptr.as_ptr() as *mut _, layout);
            }
        }
    }
}