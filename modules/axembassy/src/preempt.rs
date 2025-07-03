use alloc::collections::BTreeMap;
use axsync::Mutex;
use core::{
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
};

/// Represents a user-defined priority, ranging from 0 to 255.
type UserPrio = u8;
const MIN_USER_PRIO: UserPrio = u8::MAX;
const MAX_USER_PRIO: UserPrio = u8::MIN;

/// A fixed-point representation of priority to handle fractional values.
/// This type ensures consistent scaling and avoids magic numbers in calculations.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Prio(u64);

impl From<UserPrio> for Prio {
    fn from(value: UserPrio) -> Self {
        // Scale the user-priority to the internal fixed-point representation.
        Self((value as u64).saturating_mul(Prio::SCALING_FACTOR))
    }
}

impl From<Prio> for u64 {
    fn from(prio: Prio) -> u64 {
        prio.0
    }
}

impl Prio {
    const MAX_PRIO: Prio = Prio::raw_new(MAX_USER_PRIO as u64 * Prio::SCALING_FACTOR);
    const MIN_PRIO: Prio = Prio::raw_new(MIN_USER_PRIO as u64 * Prio::SCALING_FACTOR);
    /// The scaling factor for converting `UserPrio` to `Prio`'s internal `u64` representation.
    const SCALING_FACTOR: u64 = 100;

    /// Weights for factors in priority adjustment.
    const PRIO_EFFECT: u64 = 70;
    const ACTIVE_EFFECT: u64 = 30;

    /// Clamping range for normalized factors.
    const CLAMP_MIN: u64 = 10;
    const CLAMP_MAX: u64 = 100;

    /// Tolerance range for priority check, as a percentage of the priority.
    const TOL: u64 = 10;

    const fn raw_new(prio: u64) -> Self {
        Self(prio)
    }

    /// Returns the raw `u64` value of the priority.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Converts Prio to `UserPrio` (u8).
    ///
    /// Returns `None` if the priority overflows `UserPrio`.
    pub const fn as_user_prio(&self) -> Option<UserPrio> {
        let prio = self.0.div_euclid(Prio::SCALING_FACTOR);
        if prio > MIN_USER_PRIO as u64 {
            None
        } else {
            Some(prio as u8)
        }
    }

    /// Normalizes a current value within a given bound to a range [CLAMP_MIN_FACTOR, CLAMP_MAX_FACTOR].
    ///
    /// # Returns
    /// `(norm_pos, norm_neg)` where:
    /// - `norm_pos` indicates closeness to the lower bound (higher for values closer to low).
    /// - `norm_neg` indicates closeness to the upper bound (higher for values closer to high).
    pub fn norm_factor<F: Into<u64>, G: Into<u64>>(bound: (F, F), cur: G) -> (u64, u64) {
        let (mut low, mut high) = (bound.0.into(), bound.1.into());
        if low > high {
            core::mem::swap(&mut low, &mut high);
        }
        let cur = cur.into();

        if high == low {
            // Avoid division by zero if bounds are the same
            return (Prio::CLAMP_MAX, Prio::CLAMP_MAX);
        }

        let range = high - low;
        let norm_pos = ((cur - low).saturating_mul(Prio::CLAMP_MAX) / range)
            .clamp(Prio::CLAMP_MIN, Prio::CLAMP_MAX);
        let norm_neg = ((high - cur).saturating_mul(Prio::CLAMP_MAX) / range)
            .clamp(Prio::CLAMP_MIN, Prio::CLAMP_MAX);
        (norm_pos, norm_neg)
    }

    /// Applies a weight to a normalized factor.
    pub fn weight<F: Into<u64>, G: Into<u64>>(factor: F, weight: G) -> u64 {
        factor
            .into()
            .saturating_mul(weight.into())
            .div_euclid(Prio::CLAMP_MAX)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct FutureId(u64);

impl FutureId {
    pub fn new() -> Self {
        static ID_COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
    /// Convert the task ID to a `u64`.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

pub struct PrioFuture<F> {
    inner: F,
    id: FutureId,
    prio: Prio,
}

impl<F: Future> PrioFuture<F> {
    /// Creates a new `PrioFuture` with an initial user priority.
    pub fn new(fut: F, prio: UserPrio) -> Self {
        let id = FutureId::new();
        SCHEDULER.lock().insert(id, prio.into());
        Self {
            inner: fut,
            id,
            prio: prio.into(),
        }
    }

    /// Sets a new user priority for the future.
    pub fn set_prio(&mut self, prio: UserPrio) {
        self.prio = prio.into();
        SCHEDULER.lock().insert(self.id, prio.into());
    }
}

impl<F: Future> Future for PrioFuture<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };

        let mut s = SCHEDULER.lock();
        let cur_prio = s.cur_prio;
        let prio = this.prio;
        let tol = Prio::weight(prio, Prio::TOL);

        s.adjust_cur_prio(prio);

        // If future prio > cur_prio + tolerance, park it
        let threshold: u64 = prio.as_u64().saturating_sub(cur_prio);
        if threshold > tol {
            // info!("prio: {}, cur_prio: {}", this.prio, s.cur_prio);
            s.park_future(this.id);
            cx.waker().wake_by_ref();
            return Poll::Pending;
        } else {
            s.unpark_future(this.id);
        }

        // SAFETY: Just projecting the pin
        let inner = unsafe { Pin::new_unchecked(&mut this.inner) };
        match inner.poll(cx) {
            Poll::Ready(output) => Poll::Ready(output),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<F> Drop for PrioFuture<F> {
    fn drop(&mut self) {
        SCHEDULER.lock().remove(self.id);
    }
}

static SCHEDULER: Mutex<PrioScheduler> = Mutex::new(PrioScheduler::new());

type Active = usize;
type All = usize;
type PrioStatus = (Active, All);
type FutureInfo = (Prio, bool); // (pirority, is_active)
struct PrioScheduler {
    pub cur_prio: u64,
    tasks: BTreeMap<FutureId, FutureInfo>,
    status: BTreeMap<Prio, PrioStatus>,
}

impl PrioScheduler {
    pub const fn new() -> Self {
        Self {
            cur_prio: MIN_USER_PRIO as u64 * Prio::SCALING_FACTOR,
            tasks: BTreeMap::new(),
            status: BTreeMap::new(),
        }
    }
    /// Parks a future, marking it as inactive.
    pub fn park_future(&mut self, id: FutureId) {
        if let Some((prio, true)) = self.tasks.get(&id) {
            self.status.entry(*prio).and_modify(|(active, _)| {
                *active = active.saturating_sub(1);
            });
            self.tasks.insert(id, (*prio, false));
        }
    }
    /// Unparks a future, marking it as active.
    pub fn unpark_future(&mut self, id: FutureId) {
        if let Some((prio, false)) = self.tasks.get(&id) {
            self.status.entry(*prio).and_modify(|(active, _)| {
                *active += 1;
            });
            self.tasks.insert(id, (*prio, true));
        }
    }

    /// Return the highest and lowest priority in the active future
    ///
    /// # Output
    ///
    /// `(high, low)`
    fn prio_range(&self) -> (Prio, Prio) {
        let high = self
            .status
            .iter()
            .filter(|(_, (active, _))| *active > 0)
            .next()
            .map(|(prio, _)| *prio)
            .unwrap_or(Prio::MAX_PRIO);
        let low = self
            .status
            .iter()
            .filter(|(_, (active, _))| *active > 0)
            .last()
            .map(|(prio, _)| *prio)
            .unwrap_or(Prio::MIN_PRIO);
        (high, low)
    }

    fn get_prio_status(&self, prio: Prio) -> PrioStatus {
        self.status.get(&prio).cloned().unwrap_or((1, 1))
    }
    pub fn adjust_cur_prio(&mut self, prio: Prio) {
        let (active, all) = self.get_prio_status(prio);
        let (_, norm_active) = Prio::norm_factor((all as u64, 0), active as u64);

        let (highest, lowest) = self.prio_range();
        let (norm_prio, _) = Prio::norm_factor((highest, lowest), prio);

        let prio: u64 = prio.into();
        let cur_prio = self.cur_prio.into();
        let factor = (Prio::weight(norm_prio, Prio::PRIO_EFFECT)
            .saturating_add(Prio::weight(norm_active, Prio::ACTIVE_EFFECT)))
        .clamp(Prio::CLAMP_MIN, Prio::CLAMP_MAX);
        if prio > cur_prio {
            self.cur_prio += Prio::weight(prio - cur_prio, factor);
        } else {
            self.cur_prio -= Prio::weight(cur_prio - prio, factor);
        }
    }

    pub fn insert(&mut self, id: FutureId, prio: Prio) {
        self.status
            .entry(prio)
            .and_modify(|(active, all)| {
                *active += 1;
                *all += 1;
            })
            .or_insert((1, 1));
        self.tasks
            .entry(id)
            .and_modify(|(old_prio, _)| {
                self.status.entry(*old_prio).and_modify(|(_, cnt)| {
                    *cnt = cnt.saturating_sub(1);
                });
                *old_prio = prio;
            })
            .or_insert((prio, true));
    }

    pub fn remove(&mut self, id: FutureId) {
        let Some((prio, activated)) = self.tasks.remove(&id) else {
            return;
        };
        self.status.entry(prio).and_modify(|(active, all)| {
            if activated {
                *active = active.saturating_sub(1);
            }
            *all = all.saturating_sub(1);
        });
        self.status.retain(|_, (_, all)| *all > 0);
    }
}
