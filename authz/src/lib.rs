mod group;
mod user;

#[derive(Debug, Clone, Copy)]
pub struct Engine {
    _priv: (),
}

impl Engine {
    #[expect(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { _priv: () }
    }
}
