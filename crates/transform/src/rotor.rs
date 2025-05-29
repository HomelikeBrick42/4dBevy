use bevy::prelude::*;

#[derive(Component, Reflect, Clone, Copy)]
#[reflect(Default, Clone)]
#[require(GlobalTransform)]
pub struct Rotor;

impl Rotor {
    pub const IDENTITY: Self = Self;

    #[must_use]
    pub fn then(self, #[expect(unused)] other: Self) -> Self {
        Self {}
    }

    #[must_use]
    pub fn normalised(self) -> Self {
        Self {}
    }
}

impl Default for Rotor {
    fn default() -> Self {
        Self::IDENTITY
    }
}
