use crate::proc_anim::{DynamicBody, FabrikJoint, FabrikSync, PivotEntity, SegmentFiller};
use bevy::prelude::*;

macro_rules! spawn_basic {
    ($commands:expr, $meshes:expr, $materials:expr, $mesh:expr, $color:expr, $pos:expr) => {
        $commands.spawn((
            Mesh3d($meshes.add($mesh)),
            MeshMaterial3d($materials.add($color)),
            Transform::from_translation($pos),
        ))
    };
}

macro_rules! spawn_batch_ids {
    ($amt:expr, $commands:expr, $meshes:expr, $materials:expr, $mesh:expr, $color:expr) => {{
        let mut return_vec: Vec<Entity> = Vec::new();
        let mesh_handle = $meshes.add($mesh);
        let material_handle = $materials.add($color);

        for i in 0..$amt {
            let entity_id = $commands
                .spawn((
                    Mesh3d(mesh_handle.clone()),
                    MeshMaterial3d(material_handle.clone()),
                    Transform::from_translation(Vec3::new(i as f32, 0.0, 0.0)),
                ))
                .id();
            return_vec.push(entity_id);
        }

        return_vec
    }};
}

#[derive(Component)]
pub struct Mantis {
    pub speed: f32,
    init_center_of_mass: Vec3,
    //include color, and scale factors here later
}

impl Mantis {
    pub fn init_bundle(&self) -> impl Bundle {
        return (
            /*
            Mantis {
                speed: 5.0,
                init_center_of_mass: self.init_center_of_mass,
            },
            Mesh3d(meshes.add(Sphere::new(0.1))),
            MeshMaterial3d(materials.add(Color::srgb_u8(255, 255, 255))),
            Transform::from_xyz(center_of_mass.x, center_of_mass.y, center_of_mass.z),
             */
        );
    }
}

fn linear_downset(i: i32, prev_vec: Vec3) -> Vec3 {
    return Vec3::new(0.0, prev_vec.y - 0.1, 0.0);
}

pub fn create_mantis(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    //center of mass placeholder
    let center_of_mass = Vec3::new(0.0, 0.5, 0.0);

    let head_id = spawn_basic!(
        commands,
        meshes,
        materials,
        Sphere::new(0.1),
        Color::srgb_u8(255, 255, 255),
        center_of_mass
    )
    .insert(Mantis {
        speed: 4.0,
        init_center_of_mass: center_of_mass,
    })
    .id();

    //static head
    let static_head = spawn_basic!(
        commands,
        meshes,
        materials,
        Sphere::new(0.1),
        Color::srgb_u8(255, 255, 255),
        Vec3::ZERO
    )
    .id();
    commands.spawn(PivotEntity::new(
        head_id,
        Vec3::new(0.0, 0.1, -0.2),
        static_head,
    ));

    //create dynamic body
    let seg_lens = vec![0.2; 5];
    let segments = spawn_batch_ids!(
        seg_lens.len() + 1,
        commands,
        meshes,
        materials,
        Sphere::new(0.1),
        Color::srgb_u8(124, 144, 255)
    );
    let midpoint_segments = spawn_batch_ids!(
        seg_lens.len(),
        commands,
        meshes,
        materials,
        Cylinder::new(0.09, seg_lens[0]),
        Color::srgb_u8(255, 124, 144)
    );

    commands.spawn((
        DynamicBody::new(
            seg_lens,
            segments.clone(),
            10.0 * std::f32::consts::PI / 180.0,
            head_id,
            linear_downset,
        ),
        SegmentFiller::new(segments.clone(), midpoint_segments, Vec3::Y),
    ));

    //create fabrik joinnt
    let rad_constraints: Vec<f32> = vec![
        30.0 * std::f32::consts::PI / 180.0,
        90.0 * std::f32::consts::PI / 180.0,
        170.0 * std::f32::consts::PI / 180.0,
    ];

    let seg_lens = vec![0.2, 0.2, 0.2];
    let mut both_segments: [Vec<Entity>; 2] = [Vec::new(), Vec::new()];
    let mut both_offset_entities: [Entity; 2] = [Entity::PLACEHOLDER; 2];
    for j in 0..2 {
        for i in 0..seg_lens.len() + 1 {
            let segment_id = commands
                .spawn((
                    Mesh3d(meshes.add(Sphere::new(0.1))),
                    MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
                    Transform::from_xyz(i as f32, 0.5, 0.0),
                ))
                .id();
            both_segments[j].push(segment_id);
        }
        let offset_entity = commands
            .spawn((
                Mesh3d(meshes.add(Sphere::new(0.1))),
                MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
            ))
            .id();
        both_offset_entities[j] = offset_entity;
    }

    let mut fabriks = [Entity::PLACEHOLDER; 2];
    for i in 0..2 {
        let m = if i == 0 { 1.0 } else { -1.0 };
        let fabrik = commands
            .spawn((
                PivotEntity::new(
                    head_id,
                    Vec3::new(0.2 * m, 0.0, 0.0),
                    both_offset_entities[i],
                ),
                FabrikJoint::new_with_default(
                    seg_lens.clone(),
                    both_segments[i].clone(),
                    0.7,
                    0.2,
                    Vec3::new(0.4 * m, -0.2, -0.3),
                    both_offset_entities[i],
                    rad_constraints.clone(),
                    Vec3::new(0.0, -1.0, 0.0),
                ),
            ))
            .id();
        fabriks[i] = fabrik;
    }

    commands.spawn(FabrikSync::new_with_default(fabriks[0], fabriks[1]));
}

/*

define mantis:
abdomen is composed of dynamic body segments, on a linear downward path
legs use fabrik algorithm, later add angle restrictions, and alternating step
thorax should have breathing effect, increase rate of breathing the faster the mantis moves
thorax movement:
head should point to the direction of the mouse cursor, or as close in a dir as possible
antenae should be composed of dynamic body segments (upward curve)
pinchers should also use fabrik, different target. they should move towards the mouse


todo:
add angle restrictions to fabrik
breathing effect



*/
