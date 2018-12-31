use std::fmt::Debug;
use std::hash::Hash;
use std::mem::size_of;

use byteorder::{ByteOrder, NativeEndian};

use error::{Error, Result};

/// Return the unique identifier for a DFA's dead state in the chosen
/// representation indicated by `S`.
pub fn dead_id<S: StateID>() -> S {
    S::from_usize(0)
}

/// Check that the premultiplication of the given state identifier can fit into
/// the representation indicated by `S`. If it cannot, or if it overflows
/// `usize` itself, then an error is returned.
pub fn premultiply_overflow_error<S: StateID>(
    last_state: S,
    alphabet_len: usize,
) -> Result<()> {
    let requested_max = match last_state.to_usize().checked_mul(alphabet_len) {
        Some(requested_max) => requested_max,
        None => return Err(Error::premultiply_overflow(0, 0)),
    };
    if requested_max > S::max_id() {
        return Err(Error::premultiply_overflow(S::max_id(), requested_max));
    }
    Ok(())
}

/// Allocate the next sequential identifier for a fresh state given the
/// previously constructed state identified by `current`. If the next
/// sequential identifier would overflow `usize` or the chosen representation
/// indicated by `S`, then an error is returned.
pub fn next_state_id<S: StateID>(current: S) -> Result<S> {
    let next = match current.to_usize().checked_add(1) {
        Some(next) => next,
        None => return Err(Error::state_id_overflow(::std::usize::MAX)),
    };
    if next > S::max_id() {
        return Err(Error::state_id_overflow(S::max_id()));
    }
    Ok(S::from_usize(next))
}

/// Convert the given `usize` to the chosen state identifier representation.
/// If the given value cannot fit in the chosen representation, then an error
/// is returned.
pub fn usize_to_state_id<S: StateID>(value: usize) -> Result<S> {
    if value > S::max_id() {
        Err(Error::state_id_overflow(S::max_id()))
    } else {
        Ok(S::from_usize(value))
    }
}

/// A trait describing the representation of a DFA's state identifier.
///
/// The purpose of this trait is to safely express both the possible state
/// identifier representations that can be used in a DFA and to convert between
/// state identifier representations and types that can be used to efficiently
/// index memory (such as `usize`).
///
/// In general, one should not need to implement this trait explicitly. In
/// particular, this crate provides implementations for `u8`, `u16`, `u32`,
/// `u64` and `usize`. (`u32` and `u64` are only provided for targets that can
/// represent all corresponding values in a `usize`.)
///
/// # Safety
///
/// This trait is unsafe because the correctness of its implementations may be
/// relied upon by other unsafe code. For example, one possible way to
/// implement this trait incorrectly would be to return a maximum identifier
/// in `max_id` that is greater than the real maximum identifier. This will
/// likely result in wrap-on-overflow semantics in release mode, which can in
/// turn produce incorrect state identifiers. Those state identifiers may then
/// in turn access out-of-bounds memory in a DFA's search routine, where bounds
/// checks are explicitly elided for performance reasons.
pub unsafe trait StateID:
    Clone + Copy + Debug + Eq + Hash + PartialEq + PartialOrd + Ord
{
    /// Convert from a `usize` to this implementation's representation.
    ///
    /// Implementors may assume that `n <= Self::max_id`. That is, implementors
    /// do not need to check whether `n` can fit inside this implementation's
    /// representation.
    fn from_usize(n: usize) -> Self;

    /// Convert this implementation's representation to a `usize`.
    ///
    /// Implementors must not return a `usize` value greater than
    /// `Self::max_id` and must not permit overflow when converting between the
    /// implementor's representation and `usize`. In general, the preferred
    /// way for implementors to achieve this is to simply not provide
    /// implementations of `StateID` that cannot fit into the target platform's
    /// `usize`.
    fn to_usize(self) -> usize;

    /// Return the maximum state identifier supported by this representation.
    ///
    /// Implementors must return a correct bound. Doing otherwise may result
    /// in memory unsafety.
    fn max_id() -> usize;

    /// Read a single state identifier from the given slice of bytes in native
    /// endian format.
    ///
    /// Implementors may assume that the given slice has length at least
    /// `size_of::<Self>()`.
    fn read_bytes(slice: &[u8]) -> Self;

    /// Write this state identifier to the given slice of bytes in native
    /// endian format.
    ///
    /// Implementors may assume that the given slice has length at least
    /// `size_of::<Self>()`.
    fn write_bytes(self, slice: &mut [u8]);
}

unsafe impl StateID for usize {
    #[inline]
    fn from_usize(n: usize) -> usize { n }

    #[inline]
    fn to_usize(self) -> usize { self }

    #[inline]
    fn max_id() -> usize { ::std::usize::MAX }

    #[inline]
    fn read_bytes(slice: &[u8]) -> Self {
        NativeEndian::read_uint(slice, size_of::<usize>()) as usize
    }

    #[inline]
    fn write_bytes(self, slice: &mut [u8]) {
        NativeEndian::write_uint(slice, self as u64, size_of::<usize>())
    }
}

unsafe impl StateID for u8 {
    #[inline]
    fn from_usize(n: usize) -> u8 { n as u8 }

    #[inline]
    fn to_usize(self) -> usize { self as usize }

    #[inline]
    fn max_id() -> usize { ::std::u8::MAX as usize }

    #[inline]
    fn read_bytes(slice: &[u8]) -> Self {
        slice[0]
    }

    #[inline]
    fn write_bytes(self, slice: &mut [u8]) {
        slice[0] = self;
    }
}

unsafe impl StateID for u16 {
    #[inline]
    fn from_usize(n: usize) -> u16 { n as u16 }

    #[inline]
    fn to_usize(self) -> usize { self as usize }

    #[inline]
    fn max_id() -> usize { ::std::u16::MAX as usize }

    #[inline]
    fn read_bytes(slice: &[u8]) -> Self {
        NativeEndian::read_u16(slice)
    }

    #[inline]
    fn write_bytes(self, slice: &mut [u8]) {
        NativeEndian::write_u16(slice, self)
    }
}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
unsafe impl StateID for u32 {
    #[inline]
    fn from_usize(n: usize) -> u32 { n as u32 }

    #[inline]
    fn to_usize(self) -> usize { self as usize }

    #[inline]
    fn max_id() -> usize { ::std::u32::MAX as usize }

    #[inline]
    fn read_bytes(slice: &[u8]) -> Self {
        NativeEndian::read_u32(slice)
    }

    #[inline]
    fn write_bytes(self, slice: &mut [u8]) {
        NativeEndian::write_u32(slice, self)
    }
}

#[cfg(target_pointer_width = "64")]
unsafe impl StateID for u64 {
    #[inline]
    fn from_usize(n: usize) -> u64 { n as u64 }

    #[inline]
    fn to_usize(self) -> usize { self as usize }

    #[inline]
    fn max_id() -> usize { ::std::u64::MAX as usize }

    #[inline]
    fn read_bytes(slice: &[u8]) -> Self {
        NativeEndian::read_u64(slice)
    }

    #[inline]
    fn write_bytes(self, slice: &mut [u8]) {
        NativeEndian::write_u64(slice, self)
    }
}
