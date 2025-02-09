use nohash_hasher::IntMap;
use parking_lot::Mutex;
use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

use super::handler_id::HandlerId;

mod private {
    /// Internal type unreachable externally
    // This struct is intentionally made `!Sized` with `[()]` such that we have no overlap with
    // `Sized` arguments in specialized versions of `call_simple` implementations below
    #[derive(Debug)]
    #[allow(dead_code)]
    pub struct Private([()]);
}

struct Inner<F: Send + Sync + Clone + 'static> {
    handlers: IntMap<usize, F>,
    next_index: usize,
}

/// Data structure that holds `Fn()` event handlers
pub struct Bag<
    F: Send + Sync + Clone + 'static,
    A1: ?Sized = private::Private,
    A2: ?Sized = private::Private,
    A3: ?Sized = private::Private,
    A4: ?Sized = private::Private,
    A5: ?Sized = private::Private,
> {
    inner: Arc<Mutex<Inner<F>>>,
    a1: PhantomData<A1>,
    a2: PhantomData<A2>,
    a3: PhantomData<A3>,
    a4: PhantomData<A4>,
    a5: PhantomData<A5>,
}

impl<F, A1, A2, A3, A4, A5> fmt::Debug for Bag<F, A1, A2, A3, A4, A5>
where
    F: Send + Sync + Clone + 'static,
    A1: ?Sized,
    A2: ?Sized,
    A3: ?Sized,
    A4: ?Sized,
    A5: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bag").finish()
    }
}

impl<F, A1, A2, A3, A4, A5> Clone for Bag<F, A1, A2, A3, A4, A5>
where
    F: Send + Sync + Clone + 'static,
    A1: ?Sized,
    A2: ?Sized,
    A3: ?Sized,
    A4: ?Sized,
    A5: ?Sized,
{
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            a1: PhantomData::default(),
            a2: PhantomData::default(),
            a3: PhantomData::default(),
            a4: PhantomData::default(),
            a5: PhantomData::default(),
        }
    }
}

impl<F, A1, A2, A3, A4, A5> Default for Bag<F, A1, A2, A3, A4, A5>
where
    F: Send + Sync + Clone + 'static,
    A1: ?Sized,
    A2: ?Sized,
    A3: ?Sized,
    A4: ?Sized,
    A5: ?Sized,
{
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                handlers: IntMap::default(),
                next_index: 0,
            })),
            a1: PhantomData::default(),
            a2: PhantomData::default(),
            a3: PhantomData::default(),
            a4: PhantomData::default(),
            a5: PhantomData::default(),
        }
    }
}

impl<F, A1, A2, A3, A4, A5> Bag<F, A1, A2, A3, A4, A5>
where
    F: Send + Sync + Clone + 'static,
    A1: ?Sized,
    A2: ?Sized,
    A3: ?Sized,
    A4: ?Sized,
    A5: ?Sized,
{
    /// Add new event handler to a bag
    pub fn add(&self, callback: F) -> HandlerId {
        let index;

        {
            let mut inner = self.inner.lock();

            index = loop {
                let index = inner.next_index;
                inner.next_index += 1;

                if !inner.handlers.contains_key(&index) {
                    inner.handlers.insert(index, callback);
                    break index;
                }
            }
        }

        HandlerId::new({
            let weak_inner = Arc::downgrade(&self.inner);

            move || {
                if let Some(inner) = weak_inner.upgrade() {
                    inner.lock().handlers.remove(&index);
                }
            }
        })
    }

    /// Call applicator with each handler and keep handlers in the bag
    pub fn call<A>(&self, applicator: A)
    where
        A: Fn(&F),
    {
        // We collect handlers first in order to avoid holding lock while calling handlers
        let handlers = self
            .inner
            .lock()
            .handlers
            .values()
            .cloned()
            .collect::<Vec<F>>();
        for handler in handlers.iter() {
            applicator(handler);
        }
    }
}

impl<F: Fn() + Send + Sync + ?Sized + 'static> Bag<Arc<F>> {
    /// Call each handler without arguments and keep handlers in the bag
    #[allow(dead_code)]
    pub fn call_simple(&self) {
        self.call(|handler| handler())
    }
}

impl<A1, F> Bag<Arc<F>, A1>
where
    A1: Sized,
    F: Fn(&A1) + Send + Sync + ?Sized + 'static,
{
    /// Call each handler without arguments and keep handlers in the bag
    pub fn call_simple(&self, a1: &A1) {
        self.call(|handler| handler(a1))
    }
}

impl<A1, A2, F> Bag<Arc<F>, A1, A2>
where
    A1: Sized,
    A2: Sized,
    F: Fn(&A1, &A2) + Send + Sync + ?Sized + 'static,
{
    /// Call each handler without arguments and keep handlers in the bag
    #[allow(dead_code)]
    pub fn call_simple(&self, a1: &A1, a2: &A2) {
        self.call(|handler| handler(a1, a2))
    }
}

impl<A1, A2, A3, F> Bag<Arc<F>, A1, A2, A3>
where
    A1: Sized,
    A2: Sized,
    A3: Sized,
    F: Fn(&A1, &A2, &A3) + Send + Sync + ?Sized + 'static,
{
    /// Call each handler without arguments and keep handlers in the bag
    #[allow(dead_code)]
    pub fn call_simple(&self, a1: &A1, a2: &A2, a3: &A3) {
        self.call(|handler| handler(a1, a2, a3))
    }
}

impl<A1, A2, A3, A4, F> Bag<Arc<F>, A1, A2, A3, A4>
where
    A1: Sized,
    A2: Sized,
    A3: Sized,
    A4: Sized,
    F: Fn(&A1, &A2, &A3, &A4) + Send + Sync + ?Sized + 'static,
{
    /// Call each handler without arguments and keep handlers in the bag
    #[allow(dead_code)]
    pub fn call_simple(&self, a1: &A1, a2: &A2, a3: &A3, a4: &A4) {
        self.call(|handler| handler(a1, a2, a3, a4))
    }
}

impl<A1, A2, A3, A4, A5, F> Bag<Arc<F>, A1, A2, A3, A4, A5>
where
    A1: Sized,
    A2: Sized,
    A3: Sized,
    A4: Sized,
    A5: Sized,
    F: Fn(&A1, &A2, &A3, &A4, &A5) + Send + Sync + ?Sized + 'static,
{
    /// Call each handler without arguments and keep handlers in the bag
    #[allow(dead_code)]
    pub fn call_simple(&self, a1: &A1, a2: &A2, a3: &A3, a4: &A4, a5: &A5) {
        self.call(|handler| handler(a1, a2, a3, a4, a5))
    }
}
