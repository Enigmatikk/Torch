//! Application state extraction
//!
//! Extract shared application state from the request context.

use std::pin::Pin;
use std::future::Future;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use crate::{Request, extractors::{FromRequestParts, ExtractionError}};

/// Extract application state of a specific type
///
/// # Example
///
/// ```rust,no_run
/// use torch_web::{App, extractors::State};
/// use std::sync::Arc;
/// use tokio::sync::Mutex;
///
/// #[derive(Clone)]
/// struct AppState {
///     counter: Arc<Mutex<u64>>,
/// }
///
/// async fn increment(State(state): State<AppState>) {
///     let mut counter = state.counter.lock().await;
///     *counter += 1;
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let state = AppState {
///         counter: Arc::new(Mutex::new(0)),
///     };
///
///     let app = App::new()
///         .with_state(state)
///         .get("/increment", increment);
/// }
/// ```
pub struct State<T>(pub T);

impl<T> FromRequestParts for State<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Error = ExtractionError;

    fn from_request_parts(
        req: &mut Request,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'static>> {
        let type_id = TypeId::of::<T>();

        // Clone the state to avoid lifetime issues
        let state_result = if let Some(state_any) = req.get_state(type_id) {
            match state_any.downcast_ref::<T>() {
                Some(state) => Ok(state.clone()),
                None => Err(ExtractionError::MissingState(
                    format!("State type mismatch for {}", std::any::type_name::<T>())
                )),
            }
        } else {
            Err(ExtractionError::MissingState(
                format!("No state found for type {}", std::any::type_name::<T>())
            ))
        };

        Box::pin(async move {
            match state_result {
                Ok(state) => Ok(State(state)),
                Err(err) => Err(err),
            }
        })
    }
}

/// Container for application state
#[derive(Clone, Default)]
pub struct StateMap {
    states: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl StateMap {
    /// Create a new empty state map
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    /// Insert state of a specific type
    pub fn insert<T>(&mut self, state: T)
    where
        T: Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        self.states.insert(type_id, Arc::new(state));
    }

    /// Get state of a specific type
    pub fn get<T>(&self) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        self.states
            .get(&type_id)
            .and_then(|state| state.downcast_ref::<T>())
    }

    /// Get state by TypeId (used internally)
    pub(crate) fn get_by_type_id(&self, type_id: TypeId) -> Option<&Arc<dyn Any + Send + Sync>> {
        self.states.get(&type_id)
    }

    /// Check if state of a specific type exists
    pub fn contains<T>(&self) -> bool
    where
        T: Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        self.states.contains_key(&type_id)
    }

    /// Remove state of a specific type
    pub fn remove<T>(&mut self) -> Option<Arc<dyn Any + Send + Sync>>
    where
        T: Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        self.states.remove(&type_id)
    }

    /// Get the number of stored states
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Check if the state map is empty
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }
}

/// Extension trait for Request to handle state
pub trait RequestStateExt {
    /// Get state by TypeId
    fn get_state(&self, type_id: TypeId) -> Option<&Arc<dyn Any + Send + Sync>>;
    
    /// Set the state map for this request
    fn set_state_map(&mut self, state_map: StateMap);
    
    /// Get a reference to the state map
    fn state_map(&self) -> Option<&StateMap>;
}

// We'll implement this trait for Request in the request.rs file

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct TestState {
        value: u32,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct AnotherState {
        name: String,
    }

    #[test]
    fn test_state_map_insert_and_get() {
        let mut state_map = StateMap::new();
        
        let test_state = TestState { value: 42 };
        state_map.insert(test_state.clone());
        
        let retrieved = state_map.get::<TestState>();
        assert_eq!(retrieved, Some(&test_state));
    }

    #[test]
    fn test_state_map_multiple_types() {
        let mut state_map = StateMap::new();
        
        let test_state = TestState { value: 42 };
        let another_state = AnotherState { name: "test".to_string() };
        
        state_map.insert(test_state.clone());
        state_map.insert(another_state.clone());
        
        assert_eq!(state_map.get::<TestState>(), Some(&test_state));
        assert_eq!(state_map.get::<AnotherState>(), Some(&another_state));
    }

    #[test]
    fn test_state_map_missing_type() {
        let state_map = StateMap::new();
        let retrieved = state_map.get::<TestState>();
        assert_eq!(retrieved, None);
    }

    #[test]
    fn test_state_map_contains() {
        let mut state_map = StateMap::new();
        
        assert!(!state_map.contains::<TestState>());
        
        state_map.insert(TestState { value: 42 });
        
        assert!(state_map.contains::<TestState>());
        assert!(!state_map.contains::<AnotherState>());
    }

    #[test]
    fn test_state_map_remove() {
        let mut state_map = StateMap::new();
        
        let test_state = TestState { value: 42 };
        state_map.insert(test_state.clone());
        
        assert!(state_map.contains::<TestState>());
        
        let removed = state_map.remove::<TestState>();
        assert!(removed.is_some());
        assert!(!state_map.contains::<TestState>());
    }

    #[test]
    fn test_state_map_len_and_empty() {
        let mut state_map = StateMap::new();
        
        assert_eq!(state_map.len(), 0);
        assert!(state_map.is_empty());
        
        state_map.insert(TestState { value: 42 });
        
        assert_eq!(state_map.len(), 1);
        assert!(!state_map.is_empty());
        
        state_map.insert(AnotherState { name: "test".to_string() });
        
        assert_eq!(state_map.len(), 2);
    }
}
