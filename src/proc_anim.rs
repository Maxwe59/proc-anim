use bevy::prelude::*;

macro_rules! impl_new {
    //all manual values, you must manually fill all fields
    ($t:ty, $($field:ident : $ftype:ty),*) => {
        impl $t {
            pub fn new($($field: $ftype),*) -> Self {
                Self {
                    $($field),*
                }
            }
        }
    };

    //some default fields
    ($t:ty, [$($field:ident : $ftype:ty),*], [$($def_field:ident = $def_val:expr),*]) => {
    impl $t {
        pub fn new_with_default($($field: $ftype),*) -> Self {
            Self {
                $($field,)*
                $($def_field: $def_val),*
                }
            }
        }
    };

}

/*
Both FabrikJoint and DynamicBody assume the first segment[0] will be anchored to a "head" entity,
revolving around the head entity. (Assuming both need the component OffSetter)


*/
#[derive(Component)]
pub struct DynamicBody {
    seg_lengths: Vec<f32>, //length between segments, vec length should be seg_count - 1
    nodes: Vec<Entity>,    //vec length should be seg_count - 1
    angle_constraints: f32,
    anchor_entity: Entity,
    slope_func: fn(i32, Vec3) -> Vec3,
}

#[derive(Component)]
pub struct PivotEntity {
    head: Entity,
    offset: Vec3,
    child: Entity,
}

#[derive(Component)]
pub struct SegmentFiller {
    nodes: Vec<Entity>,
    midpoints: Vec<Entity>,
    vec_dir_segment: Vec3, //defines which vector direction the segment points in when the midpoint filler calculates rotation
}

#[derive(Component)]
pub struct FabrikJoint {
    //descriptor variables used to describe the joint behavior
    seg_lengths: Vec<f32>,
    nodes: Vec<Entity>,
    max_target_dist: f32, //max distance target (foot) can get from target_pos (global space)
    step_speed: f32,      //how fast foot lerps from one place to another
    target_offset: Vec3,  //relative to anchor position (segments[0]),
    anchor_entity: Entity, //entity the fabrik joint is anchored to.
    angle_constraints: Vec<f32>,
    init_dir_vec: Vec3, //initial vector that compares for angle constraints. relative to anchor entity

    //interal variables used to calculate states
    fabrik_iterations: i32,
    stepping: bool,        //when joint is stepping phase
    new_target_pos: Vec3, //the new foot position, when stepping is complete, curr_target becomes equal to this
    curr_target_pos: Vec3, //tracks current foot position. needed because foot doesn't move unless stepping
    t_val: f32,
    can_step: bool, //tracks if fabrik joint even steps in the first place. useful for alternating joints stepping
    just_finished_stepping: bool, //true when joint just finished stepping. has to be manually set to false to be reset
}

#[derive(Component)]
pub struct FabrikSync {
    left_joint: Entity,
    right_joint: Entity,
    current_joint: bool, //doesn't really matter what bool equals left/right, needed to distiguish between left/right
}

impl_new!(SegmentFiller, nodes: Vec<Entity>, midpoints: Vec<Entity>, vec_dir_segment: Vec3);
impl_new!(PivotEntity, head: Entity, offset: Vec3, child: Entity);
impl_new!(DynamicBody, seg_lengths: Vec<f32>, nodes: Vec<Entity>, angle_constraints: f32, anchor_entity: Entity, slope_func: fn(i32, Vec3) -> Vec3);
impl_new!(FabrikSync, [left_joint: Entity, right_joint: Entity], [current_joint = false] );

impl_new!(FabrikJoint, [
        seg_lengths: Vec<f32>,
        nodes: Vec<Entity>,
        max_target_dist: f32,
        step_speed: f32,
        target_offset: Vec3,
        anchor_entity: Entity,
        angle_constraints: Vec<f32>,
        init_dir_vec: Vec3
        ],
        
        [fabrik_iterations = 5, 
        stepping = false, 
        new_target_pos = Vec3::ZERO, 
        curr_target_pos = Vec3::ZERO, 
        t_val = 0.0, 
        can_step = true, 
        just_finished_stepping = false]);

/*
procedural animation plugin
*/
pub fn procedural_animation_plugin(app: &mut App) {
    app.add_systems(PostStartup, setup_offset).add_systems(
        Update,
        (
            dynamic_body_calculator,
            fabrik_calculator,
            fabrik_syncer,
            midpoint_filler,
        )
            .chain(),
    );
}

fn distance_restraints(pnt_static: Vec3, pnt_to_move: Vec3, distance: f32) -> Vec3 {
    let dir = (pnt_to_move - pnt_static).normalize() * distance;
    return dir + pnt_static;
}

/*
given a point_to_move and a reference point, you draw a vector from reference_vec->moved_point
and from that vector compare to a reference vector. returns new position of point_to_move if the angle
exheeds the angle_constraint parameter.
*/

fn calc_angle_constraints(
    ref_vec: Vec3,
    point_ref: Vec3,
    point_to_move: Vec3,
    angle_constraint: f32,
    segment_length: f32,
) -> Vec3 {
    let current_vec = (point_to_move - point_ref).normalize();
    let angle = ref_vec.angle_between(current_vec);
    if angle > angle_constraint {
        let axis = current_vec.cross(ref_vec).normalize();
        let new_vec = Quat::from_axis_angle(axis, angle - angle_constraint) * current_vec;
        let new_pos: Vec3 = point_ref + (new_vec * segment_length);
        return new_pos;
    } else {
        return point_to_move;
    }
}

/*
returns true if vector is within angle limits, false if the 2 vectors are too far apart angularly
*/
fn check_angle_constraint(angle_constraint: f32, vec1: Vec3, vec2: Vec3) -> bool {
    let angle = vec1.angle_between(vec2);
    if angle > angle_constraint {
        return false;
    }
    return true;
}

pub fn setup_offset(
    pivot_query: Query<&PivotEntity>,
    mut commands: Commands,
    mut transforms: Query<&mut Transform>,
) {
    for pivotter in pivot_query.iter() {
        //first set child/parent relationship
        commands
            .entity(pivotter.child)
            .insert(ChildOf(pivotter.head));
        //transform child to parent 0
        transforms.get_mut(pivotter.child).unwrap().translation = Vec3::ZERO;
        //apply offset
        transforms.get_mut(pivotter.child).unwrap().translation = pivotter.offset;
        //transforms.get_mut(pivotter.child).unwrap().translation.y += 0.5; //temporary, should be based on center of mass
    }
}

pub fn dynamic_body_calculator(
    mut transforms: Query<&mut Transform>,
    global_transforms: Query<&GlobalTransform>,
    dynamic_body_query: Query<&DynamicBody>,
) {
    for dynamic_body in dynamic_body_query.iter() {
        let anchor_entity_pos = global_transforms.get(dynamic_body.anchor_entity).unwrap();
        let nodes = &dynamic_body.nodes;
        let segment_lengths = &dynamic_body.seg_lengths;

        let mut first_node = transforms.get_mut(nodes[0]).unwrap();
        first_node.translation = anchor_entity_pos.translation();
        first_node.rotation = anchor_entity_pos.rotation();
        let mut last_vec = -1.0
            * (*global_transforms
                .get(dynamic_body.nodes[0])
                .unwrap()
                .forward());

        let mut last_node_pos = global_transforms
            .get(dynamic_body.nodes[0])
            .unwrap()
            .translation();

        let transform_func = dynamic_body.slope_func;

        for i in 0..segment_lengths.len() {
            //angle restrictions

            let front_pos = global_transforms
                .get(dynamic_body.nodes[i])
                .unwrap()
                .translation();
            let back_pos = global_transforms
                .get(dynamic_body.nodes[i + 1])
                .unwrap()
                .translation();

            let new_pos = calc_angle_constraints(
                last_vec,
                front_pos,
                back_pos,
                dynamic_body.angle_constraints,
                segment_lengths[i],
            );
            transforms.get_mut(nodes[i + 1]).unwrap().translation = new_pos;
            last_vec = (new_pos - front_pos).normalize();

            //apply segment offset
            let offset = transform_func(i as i32, last_node_pos);
            let mut node_transform = transforms.get_mut(nodes[i + 1]).unwrap();
            if offset.x != 0.0 {
                node_transform.translation.x = offset.x;
            }
            if offset.y != 0.0 {
                node_transform.translation.y = offset.y;
            }
            if offset.z != 0.0 {
                node_transform.translation.z = offset.z;
            }

            //apply distance constraints LAST
            let mut transform = transforms.get_mut(nodes[i + 1]).unwrap();
            let new_vec =
                distance_restraints(last_node_pos, transform.translation, segment_lengths[i]);
            transform.translation = new_vec;
            last_node_pos = transform.translation;
        }
    }
}

pub fn fabrik_syncer(
    mut sync_query: Query<&mut FabrikSync>,
    mut fabrik_query: Query<&mut FabrikJoint>,
) {
    for mut syncer in sync_query.iter_mut() {
        let left = syncer.left_joint;
        let right = syncer.right_joint;

        let current_stepping = if syncer.current_joint { right } else { left };

        let mut fabrik_current = fabrik_query.get_mut(current_stepping).unwrap();
        let mut next_to_step = if current_stepping == left {
            (right, false)
        } else {
            (left, false)
        };
        if fabrik_current.just_finished_stepping == true {
            fabrik_current.can_step = false;
            next_to_step.1 = true;
            fabrik_current.just_finished_stepping = false;
        }

        if next_to_step.1 {
            syncer.current_joint = !syncer.current_joint;
            let mut fabrik_next = fabrik_query.get_mut(next_to_step.0).unwrap();
            fabrik_next.can_step = true;
            fabrik_next.just_finished_stepping = false;
        }
    }
}

pub fn fabrik_calculator(
    mut fabrik_query: Query<&mut FabrikJoint>,
    mut transforms: Query<&mut Transform>,
    global_transforms: Query<&GlobalTransform>,
) {
    for mut fabrik_joint in fabrik_query.iter_mut() {
        let rotation_of_anchor = global_transforms
            .get(fabrik_joint.anchor_entity)
            .unwrap()
            .rotation();
        let updated_target = global_transforms
            .get(fabrik_joint.anchor_entity)
            .unwrap()
            .translation()
            + (rotation_of_anchor * fabrik_joint.target_offset);
        let anchor_pos = global_transforms
            .get(fabrik_joint.anchor_entity)
            .unwrap()
            .translation();

        if fabrik_joint.max_target_dist < fabrik_joint.curr_target_pos.distance(updated_target)
            && fabrik_joint.can_step
        {
            //implement lerping logic
            fabrik_joint.stepping = true;
            fabrik_joint.new_target_pos = updated_target;
            fabrik_joint.t_val = 0.0;
        }

        if fabrik_joint.stepping {
            //recalculate currentmost target (because teh entire body is moving, using old target will result in incomplete step)
            fabrik_joint.new_target_pos = updated_target;
            fabrik_joint.t_val += fabrik_joint.step_speed;
            fabrik_joint.curr_target_pos = fabrik_joint
                .curr_target_pos
                .lerp(fabrik_joint.new_target_pos, fabrik_joint.t_val);
            //reset stepping to false
            if fabrik_joint.t_val >= 1.0 {
                fabrik_joint.just_finished_stepping = true;
                fabrik_joint.stepping = false;
            }
        }

        for _i in 0..fabrik_joint.fabrik_iterations {
            //backpass
            transforms
                .get_mut(fabrik_joint.nodes.last().unwrap().clone())
                .unwrap()
                .translation = fabrik_joint.curr_target_pos;
            for i in (0..(fabrik_joint.nodes.len() - 1)).rev() {
                let prev_point = if (i + 2 >= fabrik_joint.nodes.len()) {
                    None
                } else {
                    Some(
                        transforms
                            .get(fabrik_joint.nodes[i + 2])
                            .unwrap()
                            .translation,
                    )
                };
                let point_static = transforms
                    .get(fabrik_joint.nodes[i + 1])
                    .unwrap()
                    .translation;

                let mut point_to_move: Mut<'_, Transform> =
                    transforms.get_mut(fabrik_joint.nodes[i]).unwrap();
                point_to_move.translation = distance_restraints(
                    point_static,
                    point_to_move.translation,
                    fabrik_joint.seg_lengths[i],
                );

                if let Some(point) = prev_point {
                    let prev_vec = (point_static - point).normalize();
                    let new_pos = calc_angle_constraints(
                        prev_vec,
                        point_static,
                        point_to_move.translation,
                        fabrik_joint.angle_constraints[i + 1],
                        fabrik_joint.seg_lengths[i],
                    );
                    point_to_move.translation = new_pos;
                }
            }
            //frontpass
            transforms
                .get_mut(fabrik_joint.nodes[0])
                .unwrap()
                .translation = anchor_pos;
            for i in 1..fabrik_joint.nodes.len() {
                let point_static = transforms
                    .get(fabrik_joint.nodes[i - 1])
                    .unwrap()
                    .translation;
                let ref_vec = if i == 1 {
                    rotation_of_anchor * fabrik_joint.init_dir_vec
                } else {
                    (point_static
                        - transforms
                            .get(fabrik_joint.nodes[i - 2])
                            .unwrap()
                            .translation)
                        .normalize()
                };
                let mut point_to_move = transforms.get_mut(fabrik_joint.nodes[i]).unwrap();
                point_to_move.translation = distance_restraints(
                    point_static,
                    point_to_move.translation,
                    fabrik_joint.seg_lengths[i - 1],
                );

                let new_pos = calc_angle_constraints(
                    ref_vec,
                    point_static,
                    point_to_move.translation,
                    fabrik_joint.angle_constraints[i - 1],
                    fabrik_joint.seg_lengths[i - 1],
                );
                point_to_move.translation = new_pos;
            }
        }
    }
}

fn midpoint_filler(
    segment_fillers: Query<&SegmentFiller>,
    global_transforms: Query<&GlobalTransform>,
    mut transforms: Query<&mut Transform>,
) {
    for segment_filler in segment_fillers.iter() {
        let entity_list = &segment_filler.nodes;
        let midpoint_entity_list = &segment_filler.midpoints; //will be len(entity_list)-1 length
        for i in 0..(midpoint_entity_list.len()) {
            let pos1 = global_transforms.get(entity_list[i]).unwrap().translation();
            let pos2 = global_transforms
                .get(entity_list[i + 1])
                .unwrap()
                .translation();
            let midpoint = (pos1 + pos2) / 2.0;
            let dir = (pos1 - pos2).normalize();
            let mut midpoint_entity = transforms.get_mut(midpoint_entity_list[i]).unwrap();

            //set midpoint entity to midpoint between pos1 and pos2
            midpoint_entity.translation = midpoint;

            //set rotation to be pointing in the direction of dir, moves from vector to "to" vec direction
            midpoint_entity.rotation = Quat::from_rotation_arc(segment_filler.vec_dir_segment, dir);
        }
    }
}
