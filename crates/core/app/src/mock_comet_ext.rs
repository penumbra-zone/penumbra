use crate::genesis::AppState;

pub trait BuilderExt {
    fn with_penumbra_auto_app_state(self, _: AppState) -> Self;
}

// TODO this should be implemented on `Builder` later
// TODO add a method called `penumbra_init_chain`; instead of taking the generic C, would
//      take an `App` and wrap it up. (it may need to / we may want it to be async)
impl BuilderExt for () {
    fn with_penumbra_auto_app_state(self, _: AppState) -> Self {
        // what to do here?
        // - read out list of abci/comet validators from the builder,
        // - define a penumbra validator for each one
        // - inject that into the penumbra app state
        // - serialize to json and then call `with_app_state_bytes`
        todo!()
    }
}
