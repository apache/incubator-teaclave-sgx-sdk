// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

//! Asynchronous values.

use core::cell::Cell;
use core::marker::Unpin;
use core::pin::Pin;
use core::option::Option;
use core::ptr::NonNull;
use core::task::{Context, Poll};
use core::ops::{Drop, Generator, GeneratorState};

#[doc(inline)]
pub use core::future::*;

/// Wrap a generator in a future.
///
/// This function returns a `GenFuture` underneath, but hides it in `impl Trait` to give
/// better error messages (`impl Future` rather than `GenFuture<[closure.....]>`).
pub fn from_generator<T: Generator<Yield = ()>>(x: T) -> impl Future<Output = T::Return> {
    GenFuture(x)
}

/// A wrapper around generators used to implement `Future` for `async`/`await` code.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct GenFuture<T: Generator<Yield = ()>>(T);

// We rely on the fact that async/await futures are immovable in order to create
// self-referential borrows in the underlying generator.
impl<T: Generator<Yield = ()>> !Unpin for GenFuture<T> {}

impl<T: Generator<Yield = ()>> Future for GenFuture<T> {
    type Output = T::Return;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Safe because we're !Unpin + !Drop mapping to a ?Unpin value
        let gen = unsafe { Pin::map_unchecked_mut(self, |s| &mut s.0) };
        set_task_context(cx, || match gen.resume() {
            GeneratorState::Yielded(()) => Poll::Pending,
            GeneratorState::Complete(x) => Poll::Ready(x),
        })
    }
}

thread_local! {
    static TLS_CX: Cell<Option<NonNull<Context<'static>>>> = Cell::new(None);
}

struct SetOnDrop(Option<NonNull<Context<'static>>>);

impl Drop for SetOnDrop {
    fn drop(&mut self) {
        TLS_CX.with(|tls_cx| {
            tls_cx.set(self.0.take());
        });
    }
}

/// Sets the thread-local task context used by async/await futures.
pub fn set_task_context<F, R>(cx: &mut Context<'_>, f: F) -> R
where
    F: FnOnce() -> R
{
    // transmute the context's lifetime to 'static so we can store it.
    let cx = unsafe {
        core::mem::transmute::<&mut Context<'_>, &mut Context<'static>>(cx)
    };
    let old_cx = TLS_CX.with(|tls_cx| {
        tls_cx.replace(Some(NonNull::from(cx)))
    });
    let _reset = SetOnDrop(old_cx);
    f()
}

/// Retrieves the thread-local task context used by async/await futures.
///
/// This function acquires exclusive access to the task context.
///
/// Panics if no context has been set or if the context has already been
/// retrieved by a surrounding call to get_task_context.
pub fn get_task_context<F, R>(f: F) -> R
where
    F: FnOnce(&mut Context<'_>) -> R
{
    let cx_ptr = TLS_CX.with(|tls_cx| {
        // Clear the entry so that nested `get_task_waker` calls
        // will fail or set their own value.
        tls_cx.replace(None)
    });
    let _reset = SetOnDrop(cx_ptr);

    let mut cx_ptr = cx_ptr.expect(
        "TLS Context not set. This is a rustc bug. \
        Please file an issue on https://github.com/rust-lang/rust.");

    // Safety: we've ensured exclusive access to the context by
    // removing the pointer from TLS, only to be replaced once
    // we're done with it.
    //
    // The pointer that was inserted came from an `&mut Context<'_>`,
    // so it is safe to treat as mutable.
    unsafe { f(cx_ptr.as_mut()) }
}

/// Polls a future in the current thread-local task waker.
pub fn poll_with_tls_context<F>(f: Pin<&mut F>) -> Poll<F::Output>
where
    F: Future
{
    get_task_context(|cx| F::poll(f, cx))
}
