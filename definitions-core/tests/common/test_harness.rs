// Added then_assert
//! Utility for testing a Decision implementation
//! Added then_assert which allows for more complex assertions, for eg: ignore attributes which use Utc::now
//! The test harness allows you to set up a history of events, perform the given decision,
//! and make assertions about the resulting changes.
use std::fmt::Debug;
use disintegrate::{Decision, Event, IntoState, IntoStatePart, MultiState, PersistedEvent};

/// Test harness for testing decisions.
pub struct SimpleTestHarness;

impl SimpleTestHarness {
    /// Sets up a history of events.
    ///
    /// # Arguments
    ///
    /// * `history` - A history of events to derive the current state.
    ///
    /// # Returns
    ///
    /// A `TestHarnessStep` representing the "given" step.
    pub fn given<E: Event + Clone>(history: impl Into<Vec<E>>) -> TestHarnessStep<E, Given> {
        TestHarnessStep {
            history: history.into(),
            _step: Given,
        }
    }
}

/// Represents the given step of the test harness.
pub struct Given;

/// Represents when step of the test harness.
pub struct When<R, ERR> {
    result: Result<Vec<R>, ERR>,
}

pub struct TestHarnessStep<E, ST> {
    history: Vec<E>,
    _step: ST,
}

impl<E: Event + Clone> TestHarnessStep<E, Given> {
    /// Executes a decision on the state derived from the given history.
    ///
    /// # Arguments
    ///
    /// * `decision` - The decision to test.
    ///
    /// # Returns
    ///
    /// A `TestHarnessStep` representing the "when" step.
    pub fn when<D, SP, S, ERR>(self, decision: D) -> TestHarnessStep<E, When<E, ERR>>
    where
        D: Decision<Event = E, Error = ERR, StateQuery = S>,
        S: IntoStatePart<i64, S, Target = SP>,
        SP: IntoState<S> + MultiState<i64, E>,
    {
        let mut state = decision.state_query().into_state_part();
        for event in self
            .history
            .iter()
            .enumerate()
            .map(|(id, event)| PersistedEvent::new((id + 1) as i64, event.clone()))
        {
            state.mutate_all(event);
        }
        let result = decision.process(&state.into_state());
        TestHarnessStep {
            history: self.history,
            _step: When { result },
        }
    }
}

impl<R, E, ERR> TestHarnessStep<E, When<R, ERR>>
where
    E: Event + Clone + PartialEq,
    R: Debug + PartialEq,
    ERR: Debug + PartialEq,
{
    /// Makes assertions about the changes.
    ///
    /// # Arguments
    ///
    /// * `expected` - The expected changes.
    ///
    /// # Panics
    ///
    /// Panics if the action result is not `Ok` or if the changes do not match the expected changes.
    ///
    /// # Examples
    #[track_caller]
    pub fn then(self, expected: impl Into<Vec<R>>) {
        assert_eq!(Ok(expected.into()), self._step.result);
    }

    #[track_caller]
    pub fn then_assert(self, assertion: impl FnOnce(&Vec<R>)) {
        assertion(&self._step.result.unwrap());
    }


    /// Makes assertions about the expected error result.
    ///
    /// # Arguments
    ///
    /// * `expected` - The expected error.
    ///
    /// # Panics
    ///
    /// Panics if the action result is not `Err` or if the error does not match the expected error.
    #[track_caller]
    pub fn then_err(self, expected: ERR) {
        let err = self._step.result.unwrap_err();
        assert_eq!(err, expected);
    }
}
