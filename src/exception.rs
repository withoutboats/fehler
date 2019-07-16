use std::any::TypeId;
use std::error::Error;
use std::fmt::{self, Debug, Display};
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;

pub struct Exception {
    inner: Box<InnerException<()>>,
}

impl Exception {
    pub fn new<E>(error: E) -> Exception where
        E: Error + Send + Sync + 'static
    {
        Exception::construct(error, TypeId::of::<E>())
    }

    pub fn new_adhoc<M>(message: M) -> Exception where
        M: Display + Debug + Send + Sync + 'static
    {
        Exception::construct(MessageError(message), TypeId::of::<M>())
    }

    fn construct<E>(error: E, type_id: TypeId) -> Exception where
        E: Error + Send + Sync + 'static,
    {
        unsafe {
            let obj: TraitObject = mem::transmute(&error as &dyn Error);
            let vtable = obj.vtable;
            let backtrace = Backtrace; // TODO
            let inner = InnerException { vtable, type_id, backtrace, error };
            Exception {
                inner: mem::transmute(Box::new(inner))
            }
        }
    }

    pub fn backtrace(&self) -> &Backtrace {
        &self.inner.backtrace
    }

    pub fn errors(&self) -> Errors<'_> {
        Errors { next: Some(self.inner.error()) }
    }

    pub fn is<E: Display + Debug + Send + Sync + 'static>(&self) -> bool {
        TypeId::of::<E>() == self.inner.type_id
    }

    pub fn downcast<E: Display + Debug + Send + Sync + 'static>(self) -> Result<E, Exception> {
        if let Some(error) = self.downcast_ref::<E>() {
            unsafe {
                let error = ptr::read(error);
                drop(ptr::read(&self.inner));
                mem::forget(self);
                Ok(error)
            }
        } else {
            Err(self)
        }
    }

    pub fn downcast_ref<E: Display + Debug + Send + Sync + 'static>(&self) -> Option<&E> {
        if self.is::<E>() {
            unsafe { Some(&*(self.inner.error() as *const dyn Error as *const E)) }
        } else { None }
    }

    pub fn downcast_mut<E: Display + Debug + Send + Sync + 'static>(&mut self) -> Option<&mut E> {
        if self.is::<E>() {
            unsafe { Some(&mut *(self.inner.error_mut() as *mut dyn Error as *mut E)) }
        } else { None }
    }
}

impl<E: Error + Send + Sync + 'static> From<E> for Exception {
    fn from(error: E) -> Exception {
        Exception::new(error)
    }
}

impl Deref for Exception {
    type Target = dyn Error + Send + Sync + 'static;
    fn deref(&self) -> &Self::Target {
        self.inner.error()
    }
}

impl DerefMut for Exception {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.error_mut()
    }
}

impl Debug for Exception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TODO")
    }
}

impl Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TODO")
    }
}

unsafe impl Send for Exception { }
unsafe impl Sync for Exception { }

impl Drop for Exception {
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(self.inner.error_mut()) }
    }
}

#[repr(C)]
struct InnerException<E> {
    vtable: *const (),
    type_id: TypeId,
    backtrace: Backtrace,
    error: E,
}

#[repr(C)]
struct TraitObject {
    data: *const (),
    vtable: *const (),
}

#[repr(transparent)]
struct MessageError<M: Display + Debug>(M);

impl<M: Display + Debug> Debug for MessageError<M> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<M: Display + Debug> Display for MessageError<M> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<M: Display + Debug> Error for MessageError<M> { }

impl InnerException<()> {
    fn error(&self) -> &(dyn Error + Send + Sync + 'static) {
        unsafe {
            mem::transmute(TraitObject {
                data: &self.error,
                vtable: self.vtable,
            })
        }
    }

    fn error_mut(&mut self) -> &mut (dyn Error + Send + Sync + 'static) {
        unsafe {
            mem::transmute(TraitObject {
                data: &mut self.error,
                vtable: self.vtable,
            })
        }
    }
}

pub struct Backtrace;

pub struct Errors<'a> {
    next: Option<&'a (dyn Error + 'static)>,
}

impl<'a> Iterator for Errors<'a> {
    type Item = &'a (dyn Error + 'static);
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next.take()?;
        self.next = next.source();
        Some(next)
    }
}

#[cfg(test)]
mod repr_correctness {
    use super::*;

    use std::mem;
    use std::marker::Unpin;

    #[test]
    fn size_of_exception() {
        assert_eq!(mem::size_of::<Exception>(), mem::size_of::<usize>());
    }

    #[allow(dead_code)] fn assert_exception_autotraits() where
        Exception: Unpin + Send + Sync + 'static
    { }

    #[test]
    fn destructors_work() {
        use std::sync::*;

        #[derive(Debug)] struct HasDrop(Box<Arc<Mutex<bool>>>);
        impl Error for HasDrop { }
        impl Display for HasDrop {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "does something")
            }
        }
        impl Drop for HasDrop {
            fn drop(&mut self) {
                let mut has_dropped = self.0.lock().unwrap();
                assert!(!*has_dropped);
                *has_dropped = true;
            }
        }

        let has_dropped = Arc::new(Mutex::new(false));

        drop(Exception::from(HasDrop(Box::new(has_dropped.clone()))));

        assert!(*has_dropped.lock().unwrap());

    }
}
