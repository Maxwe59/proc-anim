use std::f32::consts::PI;

use crate::helper;
use crate::proc_anim::{DynamicBody, FabrikJoint, FabrikSync, PivotEntity, SegmentFiller};
use bevy::{ecs::relationship::RelationshipSourceCollection, prelude::*};

#[derive(Component)]
pub struct CenterOfMass {
    pub speed: f32,
}

fn linear_downset(i: i32, prev_vec: Vec3) -> [Option<f32>; 3] {
    return [None, Some(prev_vec.y - 0.1), None];
}

pub fn create_mantis(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    //color constants
    let mantis_color = Color::srgb_u8(5, 237, 113);

    //center of mass placeholder
    let center_of_mass = Vec3::new(0.0, 0.5, 0.0);

    let head_id = helper::spawn_basic!(
        commands,
        center_of_mass,
        meshes,
        materials,
        Sphere::new(0.1),
        mantis_color
    )
    .insert((CenterOfMass { speed: 4.0 },))
    .id();

    //thorax
    let thorax = commands
        .spawn((
            Mesh3d(meshes.add(Cylinder::new(0.09, 0.5))),
            MeshMaterial3d(materials.add(mantis_color)),
            Transform::from_rotation(Quat::from_rotation_arc(
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 1.25, -1.0).normalize(),
            )),
        ))
        .id();

    commands.spawn(PivotEntity::new(head_id, Vec3::new(0.0, 0.2, -0.2), thorax));

    //create dynamic body
    let seg_lens = vec![0.2; 5];
    let segments = helper::spawn_batch_ids!(
        seg_lens.len() + 1,
        commands,
        meshes,
        materials,
        Sphere::new(0.01),
        Color::srgb_u8(124, 144, 255),
        true
    );
    /*
    let midpoint_segments = spawn_batch_ids!(
        seg_lens.len(),
        commands,
        meshes,
        materials,
        Cylinder::new(0.09, seg_lens[0]),
        Color::srgb_u8(255, 124, 144),
        true
    );

     */
    let mut midpoint_segments: Vec<Entity> = vec![];
    for i in 0..seg_lens.len() {
        let shape = ConicalFrustum {
            radius_top: 0.07,
            radius_bottom: 0.15,
            height: seg_lens[i],
        };
        let midseg =
            helper::spawn_basic!(commands, Vec3::ZERO, meshes, materials, shape, mantis_color).id();
        midpoint_segments.add(midseg);
    }

    let angle_constraints = vec![10.0 * std::f32::consts::PI / 180.0; seg_lens.len()];

    commands.spawn((
        DynamicBody::new(
            seg_lens,
            segments.clone(),
            angle_constraints,
            head_id,
            linear_downset,
            Vec3::new(0.0, 0.0, 1.0),
        ),
        SegmentFiller::new(segments.clone(), midpoint_segments, Vec3::Y),
    ));

    //create fabrik joint
    let rad_constraints: Vec<f32> = vec![
        30.0 * std::f32::consts::PI / 180.0,
        90.0 * std::f32::consts::PI / 180.0,
        170.0 * std::f32::consts::PI / 180.0,
    ];

    let seg_lens = vec![0.2, 0.2, 0.2];
    let mut both_segments: [Vec<Entity>; 2] = [Vec::new(), Vec::new()];
    let mut both_midpoints = [Vec::new(), Vec::new()];
    let mut both_offset_entities: [Entity; 2] = [Entity::PLACEHOLDER; 2];
    for j in 0..2 {
        both_segments[j] = helper::spawn_batch_ids!(
            seg_lens.len() + 1,
            commands,
            meshes,
            materials,
            Sphere::new(0.07),
            Color::srgb_u8(124, 144, 255),
            true
        );
        both_midpoints[j] = helper::spawn_batch_ids!(
            seg_lens.len(),
            commands,
            meshes,
            materials,
            Cylinder::new(0.07, seg_lens[0]),
            mantis_color,
            true
        );
        let offset_entity = helper::spawn_basic!(
            commands,
            Vec3::ZERO,
            meshes,
            materials,
            Sphere::new(0.07),
            Color::srgb_u8(124, 144, 255)
        )
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
                SegmentFiller::new(both_segments[i].clone(), both_midpoints[i].clone(), Vec3::Y),
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

/*
//create center of mass
//create abdomen, spawn point entities with no mesh/material, spawn segment midpoints
//create thorax. just a static offset from the center of mass (in front)
//create 2 legs, use angle restrictions to create. spawn in the points and spawn in the midpoint segs
//create static head. create antenae with dynamic body
//craete claws, try to use fabrik, if not static is fine. probably static first then figure out how to use fabrik last


*/
