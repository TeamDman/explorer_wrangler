use spacetimedb::reducer;
use spacetimedb::ReducerContext;

#[reducer(init)]
pub fn init(_ctx: &ReducerContext) {
    // Called when the module is initially published
}
