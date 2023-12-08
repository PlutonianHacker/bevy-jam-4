use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy_xpbd_3d::{
    prelude::{ColliderParent, Collisions, LinearVelocity, Position, RigidBody, Rotation, Sensor, AngularVelocity},
    SubstepSchedule, SubstepSet,
};

use crate::{GameState, projectile::Projectile, Enemy};

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CharacterAction>();

        app.add_systems(
            Update,
            (handle_input, _apply_gravity, apply_damping, apply_angular_damping, handle_character_actions)
                .chain()
                .run_if(in_state(GameState::Playing)),
        );

        app.add_systems(
            SubstepSchedule,
            handle_collisions.in_set(SubstepSet::SolveUserConstraints),
        );
    }
}

#[derive(Component, Default)]
pub struct CharacterController;

#[derive(Component)]
pub struct WalkingSpeed(pub f32);

#[derive(Component)]
pub struct RotationSpeed(pub f32);

#[derive(Component)]
pub struct PitchSpeed(pub f32);

#[derive(Component)]
pub struct DampingFactor(pub f32);

#[derive(Component)]
pub struct AngularDampingFactor(pub f32);

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct IsGrounded;

#[derive(Bundle)]
pub struct CharacterControllerBundle {
    pub character_controller: CharacterController,
    pub walking_speed: WalkingSpeed,
    pub rotation_speed: RotationSpeed,
    pub pitch_speed: PitchSpeed,
    pub damping_factor: DampingFactor,
    pub angular_damping_factor: AngularDampingFactor,
}

impl Default for CharacterControllerBundle {
    fn default() -> Self {
        Self {
            character_controller: Default::default(),
            walking_speed: WalkingSpeed(20.0),
            rotation_speed: RotationSpeed(4.0),
            pitch_speed: PitchSpeed(0.5),
            damping_factor: DampingFactor(0.92),
            angular_damping_factor: AngularDampingFactor(0.92),
        }
    }
}

#[derive(Event)]
pub enum CharacterAction {
    Turn(f32),
    Move(f32),
    Sidestep(f32),
    Pitch(f32),
}

fn handle_input(
    mut keyboard: ResMut<Input<KeyCode>>,
    mut scroll_events: EventReader<MouseWheel>,
    mut writer: EventWriter<CharacterAction>,
) {
    let w = keyboard.any_pressed([KeyCode::W, KeyCode::Up]);
    let a = keyboard.any_pressed([KeyCode::A, KeyCode::Right]);
    let s = keyboard.any_pressed([KeyCode::S, KeyCode::Left]);
    let d = keyboard.any_pressed([KeyCode::D, KeyCode::Down]);

    match (w, a, s, d) {
        (true, true, false, false) => writer.send(CharacterAction::Sidestep(-1.0)),
        (true, false, false, true) => writer.send(CharacterAction::Sidestep(1.0)),
        (true, false, false, false) => writer.send(CharacterAction::Move(-1.0)),
        (false, false, true, false) => writer.send(CharacterAction::Move(1.0)),
        (false, true, false, false) => writer.send(CharacterAction::Turn(1.0)),
        (false, false, false, true) => writer.send(CharacterAction::Turn(-1.0)),
        _ => {}
    }

    let mut scroll = 0.0;

    for event in scroll_events.read() {
        scroll += event.y;
    }

    writer.send(CharacterAction::Pitch(scroll));

    keyboard.clear();
}

fn _apply_gravity() {}

fn handle_character_actions(
    time: Res<Time>,
    mut reader: EventReader<CharacterAction>,
    mut controllers: Query<
        (&mut Transform, &WalkingSpeed, &RotationSpeed, &PitchSpeed, &mut LinearVelocity, &mut AngularVelocity),
        With<CharacterController>,
    >,
) {
    let (mut transform, walking_speed, rotation_speed, pitch_speed, mut lin_velocity, mut angular_vel) = controllers.single_mut();

    for action in reader.read() {
        match action {
            CharacterAction::Turn(angle) => {
                angular_vel.y += rotation_speed.0 * angle * time.delta_seconds();//*= Quat::from_euler(EulerRot::XYZ, 0.0, amount, 0.0);
            }
            CharacterAction::Move(dist) => {
                let (yaw, _pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);

                let x = yaw.sin() * dist;
                let z = yaw.cos() * dist;

                let direction = Vec3::new(x, 0.0, z).normalize_or_zero();

                if direction != Vec3::ZERO {
                    lin_velocity.0 += direction * walking_speed.0 * time.delta_seconds();
                }

                //transform.translation.x += walking_speed.0 * time.delta_seconds() * x * dist;
                //transform.translation.z += walking_speed.0 * time.delta_seconds() * z * dist;

                //let v = transform.rotation * (transform.rotation.inverse() * ((transform.translation + Vec3::new(0.0, 0.0, *dist)) - transform.translation));
                //transform.translation += v * time.delta_seconds() * walking_speed.0;
            }
            CharacterAction::Sidestep(dist) => {
                /*let (yaw, _pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);

                let x = yaw.cos();
                let z = yaw.sin();

                transform.translation.x += walking_speed.0 * time.delta_seconds() * x * dist;
                transform.translation.z += walking_speed.0 * time.delta_seconds() * z * dist;*/
            }
            CharacterAction::Pitch(amount) => {
                //transform.rotate_x(time.delta_seconds() * pitch_speed.0 * amount);
                let amount = pitch_speed.0 * amount * time.delta_seconds();

                transform.rotation *= Quat::from_euler(EulerRot::XYZ, amount, 0.0, 0.0);
            }
        }
    }
}

fn apply_damping(mut query: Query<(&DampingFactor, &mut LinearVelocity), Or<(With<CharacterController>, With<Enemy>)>>) {
    for (damping_factor, mut lin_vel) in query.iter_mut() {
        lin_vel.x *= damping_factor.0;
        lin_vel.z *= damping_factor.0;
    }
}

fn apply_angular_damping(mut query: Query<(&AngularDampingFactor, &mut AngularVelocity), With<CharacterController>>) {
    for (damping_factor, mut ang_vel) in query.iter_mut() {
        **ang_vel *= damping_factor.0;
    }
}

fn handle_collisions(
    collisions: Res<Collisions>,
    collider_parents: Query<&ColliderParent, (Without<Sensor>, Without<Projectile>, Without<Enemy>)>,
    mut controllers: Query<
        (&RigidBody, &mut Position, &Rotation, &mut LinearVelocity),
        With<CharacterController>,
    >,
) {
    for contacts in collisions.iter() {
        if !contacts.during_current_substep {
            continue;
        }

        let Ok([collider_parent_1, collider_parent_2]) = 
            collider_parents.get_many([contacts.entity1, contacts.entity2]) 
        else {
            continue;
        };

        let is_first: bool;
        let (rb, mut pos, rot, mut lin_vel) = if let Ok(contoller) = controllers.get_mut(collider_parent_1.get()) {
            is_first = true;
            contoller
        } else if let Ok(contoller) = controllers.get_mut(collider_parent_2.get()) {
            is_first = false;
            contoller
        } else { 
            continue 
        };

        if !rb.is_kinematic() {
            continue;
        }

        for manifold in contacts.manifolds.iter() {
            let normal = if is_first {
                -manifold.global_normal1(rot)
            } else {
                -manifold.global_normal2(rot)
            };

            for contact in manifold.contacts.iter().filter(|c| c.penetration > 0.0) {
                pos.0 += normal * contact.penetration;
            }
        }
    }
}
