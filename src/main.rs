use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::prelude::*;
use std::f32::consts::PI;

pub const PLAYER_MASS: f32 = 25.0;
pub const NUM_TILES: i32 = 50;
pub const ROAD_LENGTH: f32 = 25.0;
pub const CAR_VELOCITY: f32 = -10.0;
pub const TYPE_COLORS: [Color; 7] = [Color::rgb(0.289, 0.289, 0.289), // Gray
    Color::rgb(0.636, 0.773, 0.938), Color::rgb(0.297, 0.703, 0.39), // Blue, Green
    Color::rgb(0.289, 0.289, 0.289), Color::rgb(0.289, 0.289, 0.289),
    Color::rgb(0.289, 0.289, 0.289), Color::rgb(0.289, 0.289, 0.289)]; 

#[derive(Component)]
pub struct Score {
    current_road: i32,
}

#[derive(Component)]
pub struct Player {
    is_on_ground: bool,
}

#[derive(Component)]
pub struct Tile {
    road_type: i32,
    spawn_timer: Timer,
    left_oriented: bool,
}

#[derive(Component)]
pub struct Car {
    velocity: f32,
    despawn_timer: Timer,
    left_oriented: bool,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup_ui)
        .add_systems(Startup, setup_graphics)
        .add_systems(Startup, setup_physics)
        .add_systems(Startup, setup_light)
        //.add_systems(Update, print_ball_altitude)
        .add_systems(Update, player_jump)
        .add_systems(Update, check_player_on_ground)
        .add_systems(Update, move_tiles)
        .add_systems(Update, spawn_cars)
        .add_systems(Update, move_car)
        .add_systems(Update, update_score_text)
        .run();
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 27.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let container = NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        ..default()
    };

    let text = TextBundle::from_section (
        "Score",
            TextStyle {
                font_size: 100.0,
                color: Color::WHITE,
                ..default()
            },
        ) // Set the alignment of the Text
        .with_text_alignment(TextAlignment::Center)
        // Set the style of the TextBundle itself.
        /*.with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        })*/;

    let parent = commands.spawn(container).id();
    let child = commands.spawn(text).id();

    commands.entity(parent).push_children(&[child]);
}

fn setup_physics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    /* Create the ground. */
    for i in 0..NUM_TILES {
        let temp = (random::<f32>() * 3.0) as usize;
        let timer_duration = random::<f32>() * 3.0 + 2.0;
        commands
        .spawn(PbrBundle {
            mesh: meshes.add(shape::Box::new(ROAD_LENGTH, 1.0, 3.0).into()),
            material: materials.add(TYPE_COLORS[temp].into()),
            // transform: Transform::/*from_xyz(0.0, -2.0, 0.0)*/
            // from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            //.with_translation(Vec3::new(0.0, -2.0, 0.0)),
            //global_transform: GlobalTransform::from_xyz(0.0, -2.0, 0.0),
            ..default()
        })
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -2.0, (3 * i) as f32)))
        .insert(Collider::cuboid(12.5, 0.5, 1.5))
        .insert(Tile {
            road_type: temp as i32,
            spawn_timer: Timer::from_seconds(timer_duration, TimerMode::Repeating),
            left_oriented: rand::random(),
        });
    }
        
        /*.insert(PbrBundle {
            mesh: meshes.add(shape::Circle::new(4.0).into()),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_xyz(0.0, -2.0, 0.0)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            //.with_translation(Vec3::new(0.0, -2.0, 0.0)),
            global_transform: GlobalTransform::from_xyz(0.0, -2.0, 0.0),
            ..default()
        });*/

    /* Create the cube. */
    commands
        .spawn(RigidBody::Dynamic)
        .insert(AdditionalMassProperties::Mass(PLAYER_MASS))
        .insert(GravityScale(3.0))
        .insert(Collider::cuboid(0.75, 0.75, 1.0))
        .insert(Restitution::coefficient(0.2))
        //.insert(TransformBundle::from(Transform::from_xyz(0.0, 4.0, 0.0)))
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(1.5, 1.5, 2.0))),
            material: materials.add(Color::rgb_u8(124, 144, 255).into()),
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            global_transform: GlobalTransform::from_xyz(0.0, 2.0, 0.0),
            ..default()
        })
        .insert(ExternalImpulse {
            impulse: Vec3::new(0.0, 0.0, 0.0),
            ..default()
        })
        .insert(Velocity {
            ..default()
        })
        .insert(Player {
            is_on_ground: false,
        })
        .insert(Score {
            current_road: 0,
        });
}

fn setup_light(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 2.),
            ..default()
        },
        ..default()
    });
}

// fn print_ball_altitude(positions: Query<&Transform, With<RigidBody>>) {
//     for transform in positions.iter() {
//         println!("Ball altitude: {}", transform.translation.y);
//     }
// }

fn player_jump(
    keyboard: Res<Input<KeyCode>>,
    mut player_query: Query<(&mut Transform, &Player), With<Player>>,
    _time: Res<Time>,
) {
    for (mut player_transform, player) in &mut player_query {
        if (keyboard.any_pressed([KeyCode::Up, KeyCode::Down, KeyCode::Left, KeyCode::Right]))
            && (player.is_on_ground) {
            //println!("Jump!");
            //add Impulse (m * sqrt(2gh))
            //impulse.impulse = Vec3::new(0.0, PLAYER_MASS * ((2.0 * 9.8) as f32).sqrt(), 0.0);
            player_transform.translation.y += 0.1;
        }
    }    
}

fn move_tiles (
    keyboard: Res<Input<KeyCode>>,
    mut player_query: Query<(&mut Score, &mut Transform), (With<Player>, Without<Camera>)>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    let mut player = player_query.single_mut();
    let mut camera = camera_query.single_mut();
    if keyboard.just_pressed(KeyCode::Up) {
        player.1.translation.z += 3.0;
        camera.translation.z += 3.0;
        player.0.current_road += 1;
        println!("{}", player.0.current_road);
    } else if keyboard.just_pressed(KeyCode::Down) {
        player.1.translation.z -= 3.0;
        camera.translation.z -= 3.0;
        player.0.current_road -= 1;
        println!("{}", player.0.current_road);
    } else if keyboard.just_pressed(KeyCode::Left) {
        player.1.translation.x += 3.0;
        camera.translation.x += 3.0;
    } else if keyboard.just_pressed(KeyCode::Right) {
        player.1.translation.x -= 3.0;
        camera.translation.x -= 3.0;
    }
}

fn check_player_on_ground(
    mut player_query: Query<(&Velocity, &mut Player), With<Player>>
) {
    let (velocity, mut player) = player_query.single_mut();

    player.is_on_ground = velocity.linvel.y <= 0.1;
}

fn spawn_cars(
    mut road_query: Query<(&mut Tile, &Transform), With<Tile>>,
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mut tile, tile_transform) in &mut road_query {
        if tile.road_type == 0 {
            tile.spawn_timer.tick(time.delta());
            if tile.spawn_timer.just_finished() {
                commands.spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Box::new(4.0, 2.0, 2.5))),
                    material: materials.add(Color::RED.into()),
                    transform: match tile.left_oriented { 
                        false => Transform::from_xyz(tile_transform.translation.x - ROAD_LENGTH / 2.0,
                        1.0, tile_transform.translation.z),
                        true => Transform::from_xyz(tile_transform.translation.x + ROAD_LENGTH / 2.0,
                            1.0, tile_transform.translation.z)
                        },
                    ..default()
                })
                .insert(Car {
                    velocity: CAR_VELOCITY,
                    despawn_timer: Timer::from_seconds(10.0, TimerMode::Once),
                    left_oriented: tile.left_oriented,
                });
            }
        }    
    }
}

fn move_car(
    mut car_query: Query<(&mut Transform, &mut Car, Entity), With<Car>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (mut car_transform, mut car, entity) in &mut car_query {
        car.despawn_timer.tick(time.delta());

        if car.despawn_timer.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
        car_transform.translation.x = match car.left_oriented {
            false => car_transform.translation.x - car.velocity * time.delta_seconds(),
            true => car_transform.translation.x + car.velocity * time.delta_seconds(),
        }
    }
}

fn update_score_text(
    player_query: Query<&Score, With<Player>>,
    mut text_query: Query<&mut Text>,
) {
    let score = player_query.single();

    for mut text in &mut text_query {
        text.sections[0].value = score.current_road.to_string();
    }
}