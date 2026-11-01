use proptest::prelude::*;
use rstest::rstest;
use taloon_decisions::observation::{Observation, ObservationError};

fn record<T>(slot: &mut Option<Observation<T>>, payload: T) {
    if let Some(observation) = slot {
        observation.mark_duplicate();
    } else {
        *slot = Some(Observation::new(payload));
    }
}

#[test]
fn empty_slot_reports_missing() {
    // Arrange
    let mut slot = None::<Observation<u32>>;
    // Act
    let result = Observation::take(&mut slot);
    // Assert
    assert_eq!(result, Err(ObservationError::Missing));
}

#[rstest]
#[case::zero(0)]
#[case::maximum(u32::MAX)]
fn single_slot_returns_its_payload(#[case] payload: u32) {
    // Arrange
    let mut slot = None;
    record(&mut slot, payload);
    // Act
    let result = Observation::take(&mut slot);
    // Assert
    assert_eq!(result, Ok(payload));
}

#[test]
fn consumed_slot_reports_missing() {
    // Arrange
    let mut slot = None;
    record(&mut slot, 42);
    let _first = Observation::take(&mut slot);
    // Act
    let result = Observation::take(&mut slot);
    // Assert
    assert_eq!(result, Err(ObservationError::Missing));
}

#[test]
fn consumed_duplicate_reports_missing() {
    // Arrange
    let mut slot = None;
    record(&mut slot, 1);
    record(&mut slot, 2);
    let _first = Observation::take(&mut slot);
    // Act
    let result = Observation::take(&mut slot);
    // Assert
    assert_eq!(result, Err(ObservationError::Missing));
}

#[rstest]
#[case::two(2)]
#[case::three(3)]
#[case::many(100)]
fn repeated_observations_are_duplicate(#[case] count: usize) {
    // Arrange
    let mut slot = None;
    for payload in 0..count {
        record(&mut slot, payload);
    }
    // Act
    let result = Observation::take(&mut slot);
    // Assert
    assert_eq!(result, Err(ObservationError::Duplicate));
}

fn observe_events(events: &[Option<u32>]) -> Vec<Result<u32, ObservationError>> {
    let mut slot = None;
    events
        .iter()
        .filter_map(|event| match event {
            Some(payload) => {
                record(&mut slot, *payload);
                None
            }
            None => Some(Observation::take(&mut slot)),
        })
        .collect()
}

proptest! {
    #[test]
    fn slot_matches_independent_event_history(events in prop::collection::vec(prop::option::of(any::<u32>()), 0..100)) {
        // Arrange: segment the history independently at each consumption event.
        let takes = events.iter().filter(|event| event.is_none()).count();
        let expected: Vec<_> = events.split(Option::is_none).take(takes).map(|history| match history {
            [] => Err(ObservationError::Missing),
            [Some(payload)] => Ok(*payload),
            _ => Err(ObservationError::Duplicate),
        }).collect();
        // Act
        let actual = observe_events(&events);
        // Assert
        prop_assert_eq!(actual, expected);
    }
}

#[test]
fn retained_observation_fits_one_pointer() {
    // Arrange: BTreeMap stores occupied observations, with Missing represented by absence.
    let pointer_bytes = std::mem::size_of::<Box<u32>>();
    // Act
    let observation_bytes = std::mem::size_of::<Observation<u32>>();
    // Assert
    assert_eq!(observation_bytes, pointer_bytes);
}
