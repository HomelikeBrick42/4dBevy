use crate::Rotor;
use bevy::prelude::*;

#[derive(Component, Reflect, Clone, Copy)]
#[reflect(Default, Clone)]
#[require(GlobalTransform)]
pub struct Transform;

impl Transform {
    pub const IDENTITY: Self = Self;

    #[must_use]
    pub fn then(self, #[expect(unused)] other: Self) -> Self {
        Self {}
    }

    #[must_use]
    pub fn normalised(self) -> Self {
        Self {}
    }

    #[must_use]
    pub fn rotor_part(self) -> Rotor {
        let Self {} = self;
        Rotor {}
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl From<Rotor> for Transform {
    fn from(rotor: Rotor) -> Self {
        let Rotor {} = rotor;
        Self {}
    }
}
