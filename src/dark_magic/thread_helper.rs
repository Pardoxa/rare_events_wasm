/*
    ATTENTION!

    Currently this does not work in WebAssembly, only natively.
    Since I primarily want to run this in wasm later, 
    this needs further debugging.

    Currently the chapter1->second is trying to use this and 
    crashing instantly in the webapp (natively it runs fine).

    I suspect that either RwLock or Mutex or Arc or thread spawning is the issue.
    
    I should start by researching if wasm even supports multiple threads, this might be the core of the problem.
    If that is the case, I can probably delete this whole file as then I need to do everything in the UI thread
    anyways.

    So to debug later I should try to use those one after another and see when my app is crashing.

*/

use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, MutexGuard, RwLockWriteGuard};

use std::sync::RwLock;


#[derive(Default, Debug)]
pub enum ExposedData<T>{
    /// Was already written to
    Exists(T),
    /// Nothing written yet
    #[default]
    NotExisting
}

impl<T> From<ExposedData<T>> for Option<T>{
    fn from(value: ExposedData<T>) -> Self {
        match value{
            ExposedData::NotExisting => None,
            ExposedData::Exists(item) => Some(item)
        }
    }
}

impl<T> ExposedData<T> {
    fn take(&mut self) -> Option<T>
    {
        let old = std::mem::replace(self, ExposedData::NotExisting);
        old.into()
    }
}

#[derive(Default, Debug)]
pub struct ThreadHelper<T, E>{
    internal_data: Arc<Mutex<T>>,
    exposed_data: Arc<RwLock<ExposedData<E>>>
}

impl<T, E> ThreadHelper<T, E>
{
    /// # Internal data should never be accessed from main thread!
    /// 
    /// This lock only works when there are multiple copies.
    /// If there is only one copy, this probably means that the data 
    /// was deleted in the main thread and our helper thread should not run anymore
    pub fn internal_data_lock(&self) -> Option<MutexGuard<T>>
    {
        if self.has_multiple_copies() {
            let lock = self.internal_data.lock();
            lock.ok()
        } else {
            None
        }
    }
    
    /// This lock only works when there are multiple copies.
    /// If there is only one copy, this probably means that the data 
    /// was deleted in the main thread and our helper thread should not run anymore
    pub fn exposed_data_write_lock(&self) -> Option<RwLockWriteGuard<ExposedData<E>>>
    {
        if self.has_multiple_copies() {
            let lock = self.exposed_data.write();
            lock.ok()
        } else {
            None
        }
    }

    /// A copy of the exposed data
    /// 
    /// This is what should be accessed in the main thread to draw stuff or whatever
    pub fn exposed_data_deep_clone(&self) -> Option<E>
    where E: Clone
    {
        match self.exposed_data.read()
        { 
            Ok(reader) => {
                match reader.deref(){
                    ExposedData::Exists(e) => {
                        Some(e.clone())
                    },
                    _ =>  None
                }
            },
            _ =>  None
        }
    }

    /// Takes the exposed data, if it exists
    /// 
    /// This is what should be accessed in the main thread to draw stuff or whatever
    pub fn exposed_data_take(&self) -> Option<E>
    where E: Clone
    {
        let mut writer = self.exposed_data.write().ok()?;
        writer.deref_mut().take()
    }

    /// If only one copy of the data exists this either means 
    /// a copy for a thread worker was not spawned yet,
    /// or that the copy from the main thread went out of scope
    pub fn has_multiple_copies(&self) -> bool {
        let count = self.clone_count();
        count > 1
    }

    fn clone_count(&self) -> usize {
        Arc::strong_count(&self.internal_data)
    }

    /// Only two clones of the same data are allowed to exist at the same time!
    /// This function checks this and creates a clone
    pub fn get_clone(&self) -> Option<Self> {
        if !self.has_multiple_copies(){
            let copy = Self { 
                internal_data: self.internal_data.clone(),
                exposed_data: self.exposed_data.clone()
            };
            // there is potential for a race condition here, so we check again!
            (self.clone_count() == 2)
                .then_some(copy)
        } else {
            None
        }
    }
}
