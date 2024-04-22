// use super::raw_alloc::RawAlloc;



// #[derive(Debug)]
// #[allow(dead_code)] // it isn't dead - apparent rustc bug
// pub struct ComponentVec<T> {
//     raw: RawAlloc<T>,
//     offset: usize,
//     len: usize,
//     cap: usize,
// }

// impl<T> ComponentVec<T> {
//     fn new() -> Self {
//         Self::Loose {
//             raw: RawAlloc::new(0),
//             len: 0,
//             last_written: 0,
//         }
//     }

//     fn should_pack(&self, epoch_threshold: Epoch) -> bool {
//         match self {
//             Self::Loose { last_written, .. } => *last_written <= epoch_threshold,
//             _ => true,
//         }
//     }

//     fn as_raw_slice(&self) -> (NonNull<T>, usize) {
//         match self {
//             Self::Packed {
//                 raw, offset, len, ..
//             } => {
//                 (
//                     unsafe { NonNull::new_unchecked(raw.ptr.as_ptr().add(*offset)) },
//                     *len,
//                 )
//             }
//             Self::Loose { raw, len, .. } => {
//                 (unsafe { NonNull::new_unchecked(raw.ptr.as_ptr()) }, *len)
//             }
//         }
//     }

//     fn estimate_fragmentation(&self) -> f32 {
//         match self {
//             Self::Loose { .. } => 1f32,
//             Self::Packed { len, cap, .. } => {
//                 let empty = cap - len;
//                 f32::min(1f32, empty as f32 * size_of::<T>() as f32 / 16f32)
//             }
//         }
//     }

//     unsafe fn extend_memcopy(&mut self, epoch: Epoch, ptr: *const T, count: usize) {
//         self.ensure_capacity(epoch, count);
//         let (dst, len) = self.as_raw_slice();
//         std::ptr::copy_nonoverlapping(ptr, dst.as_ptr().add(len), count);
//         match self {
//             Self::Packed { len, .. } => *len += count,
//             Self::Loose {
//                 len, last_written, ..
//             } => {
//                 *len += count;
//                 *last_written = epoch;
//             }
//         }
//     }

//     fn ensure_capacity(&mut self, epoch: Epoch, space: usize) {
//         let (cap, len) = match self {
//             Self::Packed { cap, len, .. } => (*cap, *len),
//             Self::Loose { raw, len, .. } => (raw.cap, *len),
//         };

//         if cap - len < space {
//             self.grow(epoch, len + space);
//         }
//     }

//     fn swap_remove(&mut self, epoch: Epoch, index: usize) -> T {
//         let (ptr, len) = self.as_raw_slice();
//         assert!(len > index);

//         unsafe {
//             let item_ptr = ptr.as_ptr().add(index);
//             let last_ptr = ptr.as_ptr().add(len - 1);
//             if index < len - 1 {
//                 std::ptr::swap(item_ptr, last_ptr);
//             }
//             let value = std::ptr::read(last_ptr);
//             match self {
//                 Self::Packed { len, .. } => *len -= 1,
//                 Self::Loose {
//                     len, last_written, ..
//                 } => {
//                     *len -= 1;
//                     *last_written = epoch;
//                 }
//             }
//             value
//         }
//     }

//     fn grow(&mut self, epoch: Epoch, new_capacity: usize) {
//         debug_assert_ne!(std::mem::size_of::<T>(), 0);

//         match self {
//             Self::Packed {
//                 raw,
//                 offset,
//                 len,
//                 cap,
//             } => {
//                 // if we are currently packed, then allocate new storage and switch to loose
//                 debug_assert!(*cap < new_capacity);
//                 let new_alloc = RawAlloc::new(*len);
//                 unsafe {
//                     std::ptr::copy_nonoverlapping(
//                         raw.ptr.as_ptr().add(*offset),
//                         new_alloc.ptr.as_ptr(),
//                         *len,
//                     )
//                 };
//                 *self = Self::Loose {
//                     raw: new_alloc,
//                     len: *len,
//                     last_written: epoch,
//                 };
//             }
//             Self::Loose {
//                 raw, last_written, ..
//             } => {
//                 // if we are already free, try and resize the allocation
//                 raw.grow(new_capacity);
//                 *last_written = epoch;
//             }
//         };
//     }

//     unsafe fn pack(&mut self, dst: Rc<RawAlloc<T>>, offset: usize) {
//         let (ptr, len) = self.as_raw_slice();
//         debug_assert_ne!(std::mem::size_of::<T>(), 0);
//         debug_assert!(dst.cap >= offset + len);
//         std::ptr::copy_nonoverlapping(ptr.as_ptr(), dst.ptr.as_ptr().add(offset), len);
//         *self = Self::Packed {
//             raw: dst,
//             offset,
//             len,
//             cap: len,
//         }
//     }
// }

// impl<T> Deref for ComponentVec<T> {
//     type Target = [T];
//     fn deref(&self) -> &Self::Target {
//         let (ptr, len) = self.as_raw_slice();
//         unsafe { std::slice::from_raw_parts(ptr.as_ptr(), len) }
//     }
// }

// impl<T> DerefMut for ComponentVec<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         let (ptr, len) = self.as_raw_slice();
//         unsafe { std::slice::from_raw_parts_mut(ptr.as_ptr(), len) }
//     }
// }

// impl<T> Drop for ComponentVec<T> {
//     fn drop(&mut self) {
//         if std::mem::needs_drop::<T>() {
//             unsafe {
//                 let (ptr, len) = self.as_raw_slice();
//                 for i in 0..len {
//                     std::ptr::drop_in_place(ptr.as_ptr().add(i));
//                 }
//             }
//         }
//     }
// }