use std::alloc::{self, Layout};
use std::mem::MaybeUninit;
use std::ptr::NonNull;

pub struct Vector<T>{
    ptr: NonNull<MaybeUninit<T>>,
    capacity: usize,
    length: usize
}

impl<T> Vector<T>{
    pub fn new() -> Self{
        Self {
            ptr: NonNull::dangling(),
            capacity: 0,
            length: 0,
        }
    }

    /// Push a value onto the vector
    pub fn push(&mut self, value: T){
        if self.length == self.capacity{ // if length = capacity
            self.grow();
        }

        unsafe{
            self.ptr.as_ptr().add(self.length).write(MaybeUninit::new(value));
        }

        self.length += 1;
    }

    /// Returns the current number of elements
    pub fn size(&self) -> usize{
        self.length
    }


    fn grow(&mut self){
        let(new_capacity,new_layout) = if self.capacity == 0{
            (4, Layout::array::<MaybeUninit<T>>(4).unwrap())
        }
        else{
            let new_capacity = self.capacity * 2;
            (new_capacity, Layout::array::<MaybeUninit<T>>(new_capacity).unwrap())
        };

        let new_ptr = if self.capacity == 0{
            unsafe {alloc::alloc(new_layout)}
        }
        else{
            let old_layout = Layout::array::<MaybeUninit<T>>(self.capacity).unwrap();
            unsafe{alloc::realloc(self.ptr.as_ptr() as *mut u8,old_layout,new_layout.size())}
        };

        self.ptr = NonNull::new(new_ptr as *mut MaybeUninit<T>).unwrap_or_else(|| alloc::handle_alloc_error(new_layout));
        self.capacity = new_capacity;
    }
} // END VECTOR

impl<T> Drop for Vector<T>{
    fn drop(&mut self){
        unsafe{
            for i in 0..self.length{
                self.ptr.as_ptr().add(i).drop_in_place();
            }

            if self.capacity != 0{
                let layout = Layout::array::<MaybeUninit<T>>(self.capacity).unwrap();
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
} // END DROP

#[cfg(test)]
mod tests{
    use super::Vector;

    #[test]
    fn test_push_and_length(){
        let mut my_vec = Vector::new();
        assert_eq!(my_vec.size(), 0);

        my_vec.push(1);
        my_vec.push(2);
        my_vec.push(3);
        assert_eq!(my_vec.size(), 3);
    }

    #[test]
    fn test_grow(){
        let mut my_vec = Vector::new();

        // push more than the start capacity to trigger a growth
        for i in 0..100{
            my_vec.push(i);
        }

        assert_eq!(my_vec.size(), 100);
    }

    #[test]
    fn test_drop_safety() {
        use std::rc::Rc;
        use std::cell::RefCell;

        let counter = Rc::new(RefCell::new(0));

        struct DropCounter(Rc<RefCell<usize>>);

        impl Drop for DropCounter {
            fn drop(&mut self) {
                *self.0.borrow_mut() += 1;
            }
        }

        {
            let mut my_vec = Vec::new();
            for _ in 0..10 {
                my_vec.push(DropCounter(counter.clone()));
            }
            assert_eq!(*counter.borrow(), 0); // Nothing dropped yet
        }

        // After vec goes out of scope, all DropCounter instances are dropped
        assert_eq!(*counter.borrow(), 10);
    }
}