use avian3d::prelude::*;
use bevy::prelude::*;

use crate::level::LevelEntity;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct CubePile {
    count: u32,
    min_size: f32,
    max_size: f32,
}

pub struct ObstaclesPlugin;
impl Plugin for ObstaclesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_cube_piles);
    }
}

fn spawn_cube_piles(
    mut commands: Commands,
    piles: Query<(&CubePile, &GlobalTransform), Added<CubePile>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (pile, transform) in &piles {
        if pile.min_size <= 0.0 || pile.max_size < pile.min_size {
            warn!("CubePile needs 0 < min_size <= max_size");
            continue;
        }


        let center = transform.translation();
        let half = transform.scale();
        let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
        let material = materials.add(Color::srgb(0.9, 0.5, 0.2));

        // Each cube gets a grid cell big enough for the largest size.
        let cell = pile.max_size * 1.1;
        let columns = ((half.x * 2.0 / cell) as u32).max(1);
        let rows = ((half.z * 2.0 / cell) as u32).max(1);

        let mut cubes = Vec::with_capacity(pile.count as usize);
        
        for i in 0..pile.count {
            let column = i % columns;
            let row = (i / columns) % rows;
            let layer = i / (columns * rows);

            let position = center
                + Vec3::new(
                    -half.x + cell * (column as f32 + 0.5),
                    -half.y + cell * (layer as f32 + 0.5),
                    -half.z + cell * (row as f32 + 0.5),
                );
            let size = pile.min_size + fastrand::f32() * (pile.max_size - pile.min_size);

            cubes.push((
                LevelEntity,
                RigidBody::Dynamic,
                Collider::cuboid(1.0, 1.0, 1.0),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(position).with_scale(Vec3::splat(size)),
            ));
        }

        commands.spawn_batch(cubes);
    }
}
