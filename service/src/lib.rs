mod group;
mod user;

// TODO: authentication
#[must_use]
#[derive(Debug, Clone, Copy)]
pub struct Service(());

impl Service {
    #[expect(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(())
    }
}

pub use group::GroupRepository;
pub use user::UserRepository;
