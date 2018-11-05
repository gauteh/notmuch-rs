use std::ops::Drop;
use supercow::{Supercow, Phantomcow};

use ffi;
use utils::ToStr;
use Messages;
use MessagesOwner;
use Tags;
use TagsOwner;

pub trait ThreadOwner {}

#[derive(Debug)]
pub(crate) struct ThreadPtr {
    pub ptr: *mut ffi::notmuch_thread_t,
}

impl Drop for ThreadPtr {
    fn drop(&mut self) {
        unsafe { ffi::notmuch_thread_destroy(self.ptr) };
    }
}

#[derive(Debug)]
pub struct Thread<'o, Owner: ThreadOwner + 'o> {
    pub(crate) handle: ThreadPtr,
    marker: Phantomcow<'o, Owner>,
}

impl<'o, Owner: ThreadOwner + 'o> MessagesOwner for Thread<'o, Owner> {}
impl<'o, Owner: ThreadOwner + 'o> TagsOwner for Thread<'o, Owner> {}

impl<'o, Owner: ThreadOwner + 'o> Thread<'o, Owner> {
    pub fn from_ptr<O: Into<Phantomcow<'o, Owner>>>(
        ptr: *mut ffi::notmuch_thread_t,
        owner: O,
    ) -> Thread<'o, Owner> {
        Thread {
            handle: ThreadPtr { ptr },
            marker: owner.into(),
        }
    }

    pub fn id(self: &Self) -> String {
        let tid = unsafe { ffi::notmuch_thread_get_thread_id(self.handle.ptr) };
        tid.to_str().unwrap().to_string()
    }

    pub fn total_messages(self: &Self) -> i32 {
        unsafe { ffi::notmuch_thread_get_total_messages(self.handle.ptr) }
    }

    #[cfg(feature = "0.26")]
    pub fn total_files(self: &Self) -> i32 {
        unsafe { ffi::notmuch_thread_get_total_files(self.handle.ptr) }
    }

    pub fn toplevel_messages(self: &Self) -> Messages<Self> {
        <Self as ThreadExt<'o, Owner>>::toplevel_messages(self)
    }

    /// Get a `Messages` iterator for all messages in 'thread' in
    /// oldest-first order.
    pub fn messages(self: &Self) -> Messages<Self> {
        <Self as ThreadExt<'o, Owner>>::messages(self)
    }

    pub fn tags(&self) -> Tags<Self> {
        <Self as ThreadExt<'o, Owner>>::tags(self)
    }

    pub fn subject(self: &Self) -> String {
        let sub = unsafe { ffi::notmuch_thread_get_subject(self.handle.ptr) };

        sub.to_str().unwrap().to_string()
    }

    pub fn authors(self: &Self) -> Vec<String> {
        let athrs = unsafe { ffi::notmuch_thread_get_authors(self.handle.ptr) };

        athrs
            .to_str()
            .unwrap()
            .split(',')
            .map(|s| s.to_string())
            .collect()
    }

    /// Get the date of the oldest message in 'thread' as a time_t value.
    pub fn oldest_date(self: &Self) -> i64 {
        unsafe { ffi::notmuch_thread_get_oldest_date(self.handle.ptr) }
    }

    /// Get the date of the newest message in 'thread' as a time_t value.
    pub fn newest_date(self: &Self) -> i64 {
        unsafe { ffi::notmuch_thread_get_newest_date(self.handle.ptr) }
    }
}

pub trait ThreadExt<'o, Owner: ThreadOwner + 'o>{
    fn tags<'s, S: Into<Supercow<'s, Thread<'o, Owner>>>>(thread: S) -> Tags<'s, Thread<'o, Owner>> {
        let threadref = thread.into();
        Tags::from_ptr(
            unsafe { ffi::notmuch_thread_get_tags(threadref.handle.ptr) },
            Supercow::phantom(threadref)
        )
    }

    fn toplevel_messages<'s, S: Into<Supercow<'s, Thread<'o, Owner>>>>(thread: S) -> Messages<'s, Thread<'o, Owner>> {
        let threadref = thread.into();
        Messages::from_ptr(
            unsafe { ffi::notmuch_thread_get_toplevel_messages(threadref.handle.ptr) },
            Supercow::phantom(threadref)
        )
    }

    /// Get a `Messages` iterator for all messages in 'thread' in
    /// oldest-first order.
    fn messages<'s, S: Into<Supercow<'s, Thread<'o, Owner>>>>(thread: S) -> Messages<'s, Thread<'o, Owner>> {
        let threadref = thread.into();
        Messages::from_ptr(
            unsafe { ffi::notmuch_thread_get_messages(threadref.handle.ptr) },
            Supercow::phantom(threadref)
        )
    }

}

impl<'o, Owner: ThreadOwner + 'o> ThreadExt<'o, Owner> for Thread<'o, Owner>{
    
}

unsafe impl<'o, Owner: ThreadOwner + 'o> Send for Thread<'o, Owner> {}
unsafe impl<'o, Owner: ThreadOwner + 'o> Sync for Thread<'o, Owner> {}
