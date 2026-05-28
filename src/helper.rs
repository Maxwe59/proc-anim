use crate::proc_anim::{DynamicBody, FabrikJoint, FabrikSync, PivotEntity, SegmentFiller};
use bevy::prelude::*;

#[macro_export]
macro_rules! spawn_basic {
    ($commands:expr, $pos:expr $(, $meshes:expr, $materials:expr, $mesh:expr, $color:expr)?) => {
        $commands.spawn((
            Transform::from_translation($pos),
            $(
                Mesh3d($meshes.add($mesh)),
                MeshMaterial3d($materials.add($color)),
            )?
        ))
    };

     ($commands:expr $(, $meshes:expr, $materials:expr, $mesh:expr, $color:expr)?) => {
        $commands.spawn((
            Transform::from_translation(Vec3::ZERO),
            $(
                Mesh3d($meshes.add($mesh)),
                MeshMaterial3d($materials.add($color)),
            )?
        ))
    };
}

macro_rules! spawn_batch_ids {
    ($amt:expr, $commands:expr, $meshes:expr, $materials:expr, $mesh:expr, $color:expr, $has_spacing:expr) => {{
        let mut return_vec: Vec<Entity> = Vec::new();
        let mesh_handle = $meshes.add($mesh);
        let material_handle = $materials.add($color);

        for i in 0..$amt {
            let pos = if $has_spacing {
                Vec3::new(i as f32, 0.0, 0.0)
            } else {
                Vec3::ZERO
            };
            let entity_id = $commands
                .spawn((
                    Mesh3d(mesh_handle.clone()),
                    MeshMaterial3d(material_handle.clone()),
                    Transform::from_translation(pos),
                ))
                .id();
            return_vec.push(entity_id);
        }

        return_vec
    }};
}

pub(crate) use spawn_basic;
pub(crate) use spawn_batch_ids;
