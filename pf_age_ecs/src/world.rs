use specs::prelude as specs_prelude;
use specs::WorldExt;

pub struct World {
    inner_world: specs::World,
}

impl World {
    pub fn register_component<TypeComponent: specs_prelude::Component>(&mut self)
    where
        TypeComponent::Storage: Default,
    {
        self.inner_world.register::<TypeComponent>();
    }
}
