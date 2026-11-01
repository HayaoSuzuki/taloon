//! One request's retained cardinality, independent of identity and evidence.
#[cfg(creusot)]
use creusot_std::prelude::ensures;

/// A retained payload or an absorbing duplicate marker until consumption.
/// Absence belongs to the caller's Option/map, keeping retained entries compact.
pub enum Observation<T> {
    Single(Box<T>),
    Duplicate,
}

/// Cardinality failures cannot be mistaken for a successful observation.
#[cfg_attr(not(creusot), derive(Debug, Eq, PartialEq))]
pub enum ObservationError {
    Missing,
    Duplicate,
}

impl<T> Observation<T> {
    /// Retain the sole observation without changing its payload.
    #[must_use]
    #[cfg_attr(creusot, ensures(match result {
        Observation::Single(v) => *v == value,
        Observation::Duplicate => false,
    }))]
    pub fn new(value: T) -> Self {
        Self::Single(Box::new(value))
    }

    /// Any further observation invalidates the retained payload permanently.
    /// The caller owns disposal of the additional payload; no Box is allocated.
    #[cfg_attr(creusot, ensures((^self) == Observation::Duplicate))]
    pub fn mark_duplicate(&mut self) {
        *self = Self::Duplicate;
    }

    /// Consume an optional slot, preserving a sole payload and leaving None.
    ///
    /// # Errors
    /// Missing and duplicate observations return their distinct cardinality error.
    #[cfg_attr(creusot, ensures((^slot) == None))]
    #[cfg_attr(creusot, ensures(match *slot {
        None => result == Err(ObservationError::Missing),
        Some(Observation::Duplicate) => result == Err(ObservationError::Duplicate),
        Some(Observation::Single(v)) => result == Ok(*v),
    }))]
    pub fn take(slot: &mut Option<Self>) -> Result<T, ObservationError> {
        match slot.take() {
            None => Err(ObservationError::Missing),
            Some(Self::Duplicate) => Err(ObservationError::Duplicate),
            Some(Self::Single(value)) => Ok(*value),
        }
    }
}
