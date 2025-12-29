use std::any::*;

pub struct BoxedAnything {
    content: Box<dyn Any>,
}

impl BoxedAnything {
    pub fn new<T>(object: T) -> Self
    where
        T: 'static,
    {
        Self {
            content: Box::new(object),
        }
    }

    pub fn to_something_mut<B>(&mut self) -> Option<&mut B>
    where
        B: 'static,
    {
        self.content.downcast_mut()
    }

    pub fn to_something_or_default_mut<B>(&'_ mut self) -> &'_ mut B
    where
        B: Default + 'static,
    {
        if !(*self.content).is::<B>() {
            self.content = Box::<B>::default();
        }
        self.content.downcast_mut().unwrap()
    }

    pub fn to_something<B>(&self) -> Option<&B>
    where
        B: 'static,
    {
        self.content.downcast_ref()
    }
}
