#[macro_use]
extern crate field_offset as offset;

use std::ptr;
use std::mem;
use std::{rc, cell};

struct LinkData<LocalT, TargetT> {
    offset: offset::FieldOffset<LocalT, Link<LocalT, TargetT>>,
    target_obj: rc::Rc<cell::RefCell<TargetT>>,
    target_offset: offset::FieldOffset<TargetT, Link<TargetT, LocalT>>,
}

pub struct Link<LocalT, TargetT> {
    data: Option<LinkData<LocalT, TargetT>>,
}

impl<LocalT, TargetT> Default for Link<LocalT, TargetT> {
    fn default() -> Self {
        Link { data: None }
    }
}

impl<LocalT, TargetT> Link<LocalT, TargetT> {
    pub fn connect(first_obj: &mut rc::Rc<cell::RefCell<LocalT>>,
                   first_offset: offset::FieldOffset<LocalT, Link<LocalT, TargetT>>,
                   second_obj: &mut rc::Rc<cell::RefCell<TargetT>>,
                   second_offset: offset::FieldOffset<TargetT, Link<TargetT, LocalT>>) {
        let first_link_ptr =
            first_offset.apply_mut(&mut *first_obj.borrow_mut()) as *mut Link<LocalT, TargetT>;
        let second_link_ptr =
            second_offset.apply_mut(&mut *second_obj.borrow_mut()) as *mut Link<TargetT, LocalT>;
        assert!{first_link_ptr as usize != second_link_ptr as usize};
        unsafe {
            first_link_ptr.as_mut().unwrap().disconnect();
            second_link_ptr.as_mut().unwrap().disconnect();
            first_link_ptr.as_mut().unwrap().data = Some(LinkData {
                offset: first_offset,
                target_obj: second_obj.clone(),
                target_offset: second_offset,
            });
            second_link_ptr.as_mut().unwrap().data = Some(LinkData {
                offset: second_offset,
                target_obj: first_obj.clone(),
                target_offset: first_offset,
            })
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_null(&self) -> bool {
        self.data.is_none()
    }

    pub fn owner(&self) -> Option<&LocalT> {
        if let Some(data) = self.data.as_ref() {
            unsafe { Some(data.offset.unapply(self)) }
        } else {
            None
        }
    }

    pub fn owner_mut(&mut self) -> Option<&mut LocalT> {
        let self_mut_ptr = self as *mut _;
        if let Some(data) = self.data.as_ref() {
            unsafe { Some(data.offset.unapply_mut(self_mut_ptr.as_mut().unwrap())) }
        } else {
            None
        }
    }

    pub fn owner_ptr(&self) -> *const LocalT {
        self.owner().map_or(ptr::null(), |r| r as *const _)
    }

    pub fn owner_mut_ptr(&mut self) -> *mut LocalT {
        self.owner_mut().map_or(ptr::null_mut(), |r| r as *mut _)
    }

    pub fn remote_owner(&self) -> Option<&TargetT> {
        if let Some(data) = self.data.as_ref() {
            unsafe { Some((&*(*data.target_obj).borrow() as *const _).as_ref().unwrap()) }
        } else {
            None
        }
    }

    pub fn remote_owner_mut(&mut self) -> Option<&mut TargetT> {
        if let Some(data) = self.data.as_ref() {
            unsafe { Some((&mut *(*data.target_obj).borrow_mut() as *mut _).as_mut().unwrap()) }
        } else {
            None
        }
    }

    pub fn remote_owner_ptr(&self) -> *const TargetT {
        self.remote_owner().map_or(ptr::null(), |r| r as *const _)
    }

    pub fn remote_owner_mut_ptr(&mut self) -> *mut TargetT {
        self.remote_owner_mut().map_or(ptr::null_mut(), |r| r as *mut _)
    }

    pub fn swap(&mut self, other: &mut Self) {
        {
            let self_is_null = self.is_null();
            let other_is_null = other.is_null();
            if self_is_null && other_is_null {
                return;
            }
            assert!(!self_is_null && !other_is_null);
            assert!(self as *mut _ != other as *mut _);
        }
        mem::swap(&mut self.data.as_mut().unwrap().target_obj,
                  &mut other.data.as_mut().unwrap().target_obj);
        mem::swap(&mut self.data.as_mut().unwrap().target_offset,
                  &mut other.data.as_mut().unwrap().target_offset);
    }

    pub fn disconnect(&mut self) {
        if let Some(data) = self.data.as_mut() {
            let mut target_mut = data.target_obj
                .borrow_mut();
            let target_link = data.target_offset
                .apply_mut(&mut *target_mut);
            target_link.data = None;
        }
        self.data = None;
    }
}


#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::cell::RefCell;
    use Link;

    #[test]
    fn test_owned() {
        struct A {
            pub data: u32,
            pub link: Link<A, B>,
        }
        struct B {
            pub data: String,
            pub link: Link<B, A>,
        }

        let mut a = Rc::new(RefCell::new(A {
            data: 42,
            link: Link::new(),
        }));
        assert!(a.borrow().link.is_null());
        if true {
            let mut b = Rc::new(RefCell::new(B {
                data: "hello".to_owned(),
                link: Link::new(),
            }));
            Link::connect(&mut a, offset_of!{A => link}, &mut b, offset_of!{B => link});
            assert!(!a.borrow().link.is_null());
            assert!(!b.borrow().link.is_null());
            assert_eq!(a.borrow().link.owner_ptr(), &*a.borrow() as *const A);
            assert_eq!(b.borrow().link.owner_ptr(), &*b.borrow() as *const B);
            a.borrow_mut().link.disconnect();
            assert!(a.borrow().link.is_null());
            assert!(b.borrow().link.is_null());
            Link::connect(&mut b, offset_of!{B => link}, &mut a, offset_of!{A => link});
            assert!(!a.borrow().link.is_null());
            assert!(!b.borrow().link.is_null());
            assert_eq!(a.borrow().link.owner_ptr(), &*a.borrow() as *const A);
            assert_eq!(b.borrow().link.owner_ptr(), &*b.borrow() as *const B);
            b.borrow_mut().link.disconnect();
        }
        assert!(a.borrow().link.is_null());
    }
}
