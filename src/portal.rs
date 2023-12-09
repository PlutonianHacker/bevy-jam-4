use bevy::prelude::*;

use crate::GameState;

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Portal>()
            .register_type::<PortalId>()
            .add_systems(OnEnter(GameState::Playing), position_projectors)
            .add_systems(OnEnter(GameState::Playing), show_projector_hit)
            .add_systems(Update, update_hit_timers.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Portal;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct PortalId(pub u32);

#[derive(Component)]
pub struct Projector;

#[derive(Component)]
pub struct HitTimer(pub Timer, pub Handle<StandardMaterial>);

fn position_projectors(
    portals: Query<&Transform, With<Portal>>,
    mut projectors: Query<&mut Transform, (With<Projector>, Without<Portal>)>,
) {
    //let portal_positions = portals.iter().map(|t| t.translation.distance());

    for (mut proj_transform) in projectors.iter_mut() {
        let nearest_portal = portals
            .iter()
            .filter(|t| t.translation.distance(proj_transform.translation) <= 20.0)
            .collect::<Vec<_>>()[0];

        let d = nearest_portal.translation - proj_transform.translation;
        let angle = d.z.atan2(d.x);

        proj_transform.look_at(
            Vec3::new(
                nearest_portal.translation.x,
                0.0,
                nearest_portal.translation.y,
            ),
            Vec3::Y,
        );
    }
}

fn show_projector_hit(
    mut commands: Commands,
    parents: Query<Entity, With<Projector>>,
    children: Query<&Children>,
    material_handles: Query<&Handle<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in parents.iter() {
        for descendant in children.iter_descendants(entity) {
            if let Ok(handle) = material_handles.get(descendant) {
                let mat = materials.get_mut(handle).unwrap();
                //let color = mat.base_color.clone();

                //println!("color: {:?}", mat.base_color);
                
                commands.entity(descendant).insert(HitTimer(
                    Timer::from_seconds(1.0, TimerMode::Once),
                    handle.clone(),
                ));

                //*mat = Color::RED.into(); 

                //mat.base_color = Color::BLUE;
            }
        }
    }
}

fn update_hit_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut timers: Query<(Entity, &mut HitTimer, &mut Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut timer, mut handle) in timers.iter_mut() {
        if timer.0.tick(time.delta()).finished() {
            //let mat = materials.get_mut(handle).unwrap();

            //println!("{:?}", timer.1);

            *handle = timer.1.clone();

            //mat.base_color = timer.1;

            commands.entity(entity).remove::<HitTimer>();//.insert(timer.1.clone());
        }
    }
}
