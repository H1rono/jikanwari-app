mod group;
mod user;

// TODO: authentication
#[must_use]
#[derive(Debug, Clone, Copy)]
pub struct Service(());

pub use group::GroupRepository;
pub use user::UserRepository;
