use spacetimedb::SpacetimeType;

#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum TaskbarRemoteKind {
    Windows,
    FakeData,
}
