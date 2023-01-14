use bevy::prelude::*;

pub const HEIGHT: f32 = 1080.0;
pub const WIDTH: f32 = 2560.0;

fn main() {
    App::new()
    .insert_resource(ClearColor(Color::rgb(0.0,0.0,0.0)))
    .add_startup_system(spawn_player_tank)
    .add_system(player_tank_controls)
    .add_system(bullet_movement)
    .add_system(bullet_despawn)
    .add_startup_system(spawn_level)
    .add_startup_system(spawn_camera)
    .add_startup_system(spawn_enemy_tank)
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        window: WindowDescriptor {
            width: WIDTH,
            height: HEIGHT,
            title: "My Bevy Game".to_string(),
             resizable: false,
            ..default()
        },
        ..default()
    }))
    .run();
}

#[derive(Component,Reflect)]
pub struct Hitbox {
    dimensions: Vec3
}

#[derive(Component,Reflect,Debug)]
pub struct Player {
    health: f32,
}

#[derive(Component,Reflect,Debug)]
pub struct Enemy {
    health: f32,
}

#[derive(Component,Reflect)]
pub struct Bullet {
    lifetime: Timer,
    speed: f32,
}

fn bullet_despawn(
    mut commands: Commands,
    mut bullets: Query<(Entity, &mut Bullet)>,
    time: Res<Time>,
) {
    for (entity, mut bullet )in &mut bullets {
        bullet.lifetime.tick(time.delta());
        if bullet.lifetime.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}


fn spawn_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {size: 50.0})),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    })
    .insert(Name::new("Ground"));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance:25000.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
        ..default()
    });

}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Z),
        ..default()
    });
}

fn spawn_player_tank(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube {size: 1.0})),
        material: materials.add(Color::rgb(0.67, 0.84, 0.92).into()),
        transform: Transform::from_xyz(5.0,0.5,0.0),
        ..default()
    })
    .insert(Player { health: 100.0 })
    .insert(Hitbox {
        dimensions: Vec3::new(1.0, 1.0, 1.0)
    });
}

fn spawn_enemy_tank(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube {size: 1.0})),
        material: materials.add(Color::rgb(0.67, 0.84, 0.92).into()),
        transform: Transform::from_xyz(-5.0,0.5,0.0),
        ..default()
    })
    .insert(Enemy { health: 100.0 })
    .insert(Hitbox {
        dimensions: Vec3::new(1.0, 1.0, 1.0)
    });
}

fn player_tank_controls(
    mut keys: Res<Input<KeyCode>>,
    mut playertank: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    for mut transform in &mut playertank {
        if keys.pressed(KeyCode::W) {
            let forward = transform.rotation * Vec3::Z * -1.0;
            transform.translation += forward * time.delta_seconds();
        }
        if keys.pressed(KeyCode::A) {
            transform.rotate(Quat::from_rotation_y(time.delta_seconds() * 1.0));
            
        }
        if keys.pressed(KeyCode::D) {
            transform.rotate(Quat::from_rotation_y(time.delta_seconds() * -1.0));
        }
        if keys.pressed(KeyCode::S) {
            let forward = transform.rotation * Vec3::Z * 1.0;
            transform.translation += forward * time.delta_seconds();
        }
        if keys.just_pressed(KeyCode::Space) {
            
            let offset = Vec3::new(0.0, 0.0, -(1.0/2.0+0.052));
            let spawn_location = transform.translation + offset;
            let spawn_transform =    Transform::from_translation(spawn_location).with_rotation(transform.rotation);
            commands.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube {size: 0.1})),
                material: materials.add(Color::rgb(0.87, 0.44, 0.42).into()),
                transform: spawn_transform,
                ..default()
            })
            .insert(Hitbox {
                dimensions: Vec3::new(0.1, 0.1, 0.1)
            })
            .insert(Bullet {
                lifetime: Timer::from_seconds(2.0, TimerMode::Once),
                speed: 5.0
            });
        }
    } 
}



fn bullet_movement(
    mut commands: Commands,
    time: Res<Time>,
    mut bullets: Query<(Entity, &Bullet, &Hitbox, &mut Transform)>,
    mut enemy_hitboxes: Query<(&Hitbox, Entity, &Transform, &mut Enemy), Without<Bullet>>,
) {
    for (bullet_entity, bullet, bullet_hitbox, mut transform)in &mut bullets {
        let forwards = transform.rotation * Vec3::Z * -bullet.speed;
        transform.translation += forwards * time.delta_seconds();
        for (hitbox, entity, location, mut enemy) in &mut enemy_hitboxes {
            let location_delta = transform.translation-location.translation;
            if  location_delta.z.abs() <= hitbox.dimensions.z/2.0+bullet_hitbox.dimensions.z/2.0+bullet.speed*time.delta_seconds() && location_delta.x.abs() <= hitbox.dimensions.x/2.0+bullet_hitbox.dimensions.x/2.0+bullet.speed*time.delta_seconds() && location_delta.y.abs() <= hitbox.dimensions.y/2.0+bullet_hitbox.dimensions.y/2.0+bullet.speed*time.delta_seconds()  {
                commands.entity(bullet_entity).despawn_recursive();
                enemy.health -= 5.0;
                println!("{:?}", enemy.health);
                if enemy.health <= 0.0 {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}