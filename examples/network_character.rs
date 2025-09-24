use avian3d::prelude::*;
use bevy::prelude::*;
use clap::Parser;
use leafwing_input_manager::prelude::*;
use lightyear::{
    netcode::{Key, NetcodeClient, NetcodeServer},
    prelude::{
        client::ClientPlugins,
        server::{ClientOf, ServerPlugins, ServerUdpIo, Start},
        *,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};
use survicraft::prelude::*;

#[derive(Parser)]
#[command(name = "survicraft-network-character")]
#[command(version = "0.1")]
#[command(about = "Example for the networked survicraft character controller", long_about = None)]
struct Cli {
    #[arg(short = 'H', long = "host", default_value_t = false)]
    host: bool,
}

fn main() {
    let cli = Cli::parse();
    let is_host = cli.host;

    let mut app = new_gui_app();

    app.add_plugins(ClientPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
    });
    if is_host {
        app.add_plugins(ServerPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        });
    }

    app.add_plugins(ProtocolPlugin);
    app.register_component::<FloorMarker>()
        .add_prediction(PredictionMode::Once)
        .add_interpolation(InterpolationMode::Once);
    app.register_component::<WallMarker>()
        .add_prediction(PredictionMode::Once)
        .add_interpolation(InterpolationMode::Once);
    app.register_component::<CubeMarker>()
        .add_prediction(PredictionMode::Once)
        .add_interpolation(InterpolationMode::Once);

    app.add_plugins(CommonRendererPlugin);
    app.add_plugins(
        PhysicsPlugins::default()
            .build()
            // disable Sync as it is handled by lightyear_avian
            .disable::<SyncPlugin>()
            // interpolation is handled by lightyear_frame_interpolation
            .disable::<PhysicsInterpolationPlugin>()
            // disable Sleeping plugin as it can mess up physics rollbacks
            .disable::<SleepingPlugin>(),
    );

    app.add_plugins(NetworkPlayerControllerPlugin { render: true });

    app.add_systems(Startup, setup_render);
    if is_host {
        app.add_systems(Startup, setup_world);
        app.add_systems(Startup, setup_server);
    } else {
        app.add_systems(Startup, setup_client);
        app.add_systems(
            Update,
            (add_floor_to_client, add_wall_to_client, add_cube_to_client),
        );
    }
    app.add_systems(
        Update,
        (add_floor_cosmetic, add_wall_cosmetic, add_cube_cosmetic),
    );

    if is_host {
        app.add_observer(on_new_client);
        app.add_observer(on_new_connection);
    }

    app.run();
}

#[derive(Serialize, Deserialize, Component, Debug, Clone, PartialEq, Eq)]
struct FloorMarker;

#[derive(Serialize, Deserialize, Component, Debug, Clone, PartialEq, Eq)]
struct WallMarker;

#[derive(Serialize, Deserialize, Component, Debug, Clone, PartialEq, Eq)]
struct CubeMarker;

fn add_floor_cosmetic(
    mut commands: Commands,
    q_floor: Query<Entity, (With<FloorMarker>, Without<Mesh3d>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in q_floor.iter() {
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Cuboid::new(20.0, 1.0, 20.0))),
            MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
        ));
    }
}

fn add_floor_to_client(
    mut commands: Commands,
    q_floor: Query<Entity, (With<FloorMarker>, Without<Collider>)>,
) {
    for entity in q_floor.iter() {
        commands
            .entity(entity)
            .insert((Collider::cuboid(20.0, 1.0, 20.0), RigidBody::Static));
    }
}

fn add_wall_cosmetic(
    mut commands: Commands,
    q_wall: Query<Entity, (With<WallMarker>, Without<Mesh3d>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let wall_thickness = 1.0;
    let wall_height = 10.0;
    let wall_length = 20.0 * 2.0 + wall_thickness * 2.0;
    let wall_color = Color::srgb(0.4, 0.4, 0.4);
    let wall_material = materials.add(StandardMaterial {
        base_color: wall_color,
        ..default()
    });
    for entity in q_wall.iter() {
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Cuboid::new(wall_thickness, wall_height, wall_length))),
            MeshMaterial3d(wall_material.clone()),
        ));
    }
}

fn add_wall_to_client(
    mut commands: Commands,
    q_wall: Query<Entity, (With<WallMarker>, Without<Collider>)>,
) {
    let wall_thickness = 1.0;
    let wall_height = 10.0;
    let wall_length = 20.0 * 2.0 + wall_thickness * 2.0;
    for entity in q_wall.iter() {
        commands.entity(entity).insert((
            Collider::cuboid(wall_thickness, wall_height, wall_length),
            RigidBody::Static,
        ));
    }
}

fn add_cube_to_client(
    mut commands: Commands,
    q_cube: Query<Entity, (With<CubeMarker>, Without<Collider>)>,
) {
    for entity in q_cube.iter() {
        commands
            .entity(entity)
            .insert((Collider::cuboid(0.5, 0.5, 0.5), RigidBody::Dynamic));
    }
}

fn add_cube_cosmetic(
    mut commands: Commands,
    q_cube: Query<
        Entity,
        (
            Or<(Added<Predicted>, Added<Replicate>)>,
            With<CubeMarker>,
            Without<Mesh3d>,
        ),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in q_cube.iter() {
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
        ));
    }
}

fn setup_render(mut commands: Commands) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(60.0, 60.0, 00.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Directional Light"),
    ));
}

fn setup_world(mut commands: Commands) {
    const FLOOR_WIDTH: f32 = 20.0;
    const FLOOR_HEIGHT: f32 = 1.0;

    commands.spawn((
        Name::new("Floor"),
        FloorMarker,
        Collider::cuboid(FLOOR_WIDTH, FLOOR_HEIGHT, FLOOR_WIDTH),
        RigidBody::Static,
        Position::new(Vec3::ZERO),
        Replicate::to_clients(NetworkTarget::All),
    ));

    commands.spawn((
        Name::new("Ramp"),
        FloorMarker,
        Collider::cuboid(FLOOR_WIDTH, FLOOR_HEIGHT, FLOOR_WIDTH),
        RigidBody::Static,
        Position::new(Vec3::new(
            -5.0,
            FLOOR_HEIGHT * std::f32::consts::FRAC_1_SQRT_2,
            0.0,
        )),
        Rotation::from(Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            0.0,
            0.0,
            -std::f32::consts::FRAC_PI_4,
        ))),
        Replicate::to_clients(NetworkTarget::All),
    ));

    for i in 0..5 {
        commands.spawn((
            Name::new(format!("Cube {i}")),
            CubeMarker,
            Collider::cuboid(0.5, 0.5, 0.5),
            RigidBody::Dynamic,
            Position::new(Vec3::new(i as f32 - 2.0, 5.0 + i as f32, 0.0)),
            Replicate::to_clients(NetworkTarget::All),
            PredictionTarget::to_clients(NetworkTarget::All),
        ));
    }

    // for i in 0..5 {
    //     commands.spawn((
    //         Name::new(format!("Sphere {i}")),
    //         Collider::sphere(0.5),
    //         RigidBody::Dynamic,
    //         Position::new(Vec3::new(i as f32 - 2.0, 5.0 + i as f32, 2.0)),
    //         Mesh3d(meshes.add(Sphere::new(0.5))),
    //         MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.8))),
    //     ));
    // }

    let wall_thickness = 1.0;
    let wall_height = 10.0;
    let wall_length = FLOOR_WIDTH * 2.0 + wall_thickness * 2.0;
    commands.spawn((
        Name::new("Wall +X"),
        WallMarker,
        Collider::cuboid(wall_thickness, wall_height, wall_length),
        RigidBody::Static,
        Position::new(Vec3::new(
            FLOOR_WIDTH / 2.0 + wall_thickness,
            wall_height / 2.0,
            0.0,
        )),
        Rotation::default(),
        Replicate::to_clients(NetworkTarget::All),
    ));
    commands.spawn((
        Name::new("Wall -X"),
        WallMarker,
        Collider::cuboid(wall_thickness, wall_height, wall_length),
        RigidBody::Static,
        Position::new(Vec3::new(
            -FLOOR_WIDTH / 2.0 - wall_thickness,
            wall_height / 2.0,
            0.0,
        )),
        Rotation::default(),
        Replicate::to_clients(NetworkTarget::All),
    ));
    commands.spawn((
        Name::new("Wall +Z"),
        WallMarker,
        Collider::cuboid(wall_thickness, wall_height, wall_length),
        RigidBody::Static,
        Position::new(Vec3::new(
            0.0,
            wall_height / 2.0,
            FLOOR_WIDTH / 2.0 + wall_thickness,
        )),
        Rotation::from(Quat::from_euler(
            EulerRot::XYZ,
            0.0,
            std::f32::consts::FRAC_PI_2,
            0.0,
        )),
        Replicate::to_clients(NetworkTarget::All),
    ));
    commands.spawn((
        Name::new("Wall -Z"),
        WallMarker,
        Collider::cuboid(wall_thickness, wall_height, wall_length),
        RigidBody::Static,
        Position::new(Vec3::new(
            0.0,
            wall_height / 2.0,
            -FLOOR_WIDTH / 2.0 - wall_thickness,
        )),
        Rotation::from(Quat::from_euler(
            EulerRot::XYZ,
            0.0,
            std::f32::consts::FRAC_PI_2,
            0.0,
        )),
        Replicate::to_clients(NetworkTarget::All),
    ));
}

fn setup_server(mut commands: Commands) {
    info!("Starting server on {}", SERVER_ADDR);

    let server = commands
        .spawn((
            Name::new("Server"),
            NetcodeServer::new(server::NetcodeConfig::default().with_protocol_id(PROTOCOL_ID)),
            LocalAddr(SERVER_ADDR),
            ServerUdpIo::default(),
        ))
        .id();

    let client = commands
        .spawn((
            Name::new("Host Client"),
            Client::default(),
            LinkOf { server },
        ))
        .id();

    commands.trigger_targets(Start, server);
    commands.trigger_targets(Connect, client);
}

fn setup_client(mut commands: Commands) {
    let auth = Authentication::Manual {
        server_addr: SERVER_ADDR,
        client_id: get_client_id(),
        private_key: Key::default(),
        protocol_id: PROTOCOL_ID,
    };

    let conditioner = LinkConditionerConfig::average_condition();
    let client = commands
        .spawn((
            Name::new("Client"),
            Client::default(),
            Link::new(Some(RecvLinkConditioner::new(conditioner.clone()))),
            LocalAddr(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)),
            PeerAddr(SERVER_ADDR),
            ReplicationReceiver::default(),
            PredictionManager::default(),
            InterpolationManager::default(),
            NetcodeClient::new(auth, client::NetcodeConfig::default()).unwrap(),
            UdpIo::default(),
        ))
        .insert(ReplicationSender::new(
            SERVER_REPLICATION_INTERVAL,
            SendUpdatesMode::SinceLastAck,
            false,
        ))
        .id();

    commands.trigger_targets(Connect, client);
}

fn on_new_client(
    trigger: Trigger<OnAdd, LinkOf>,
    mut commands: Commands,
    _server: Single<&Server>,
) {
    info!("New client connected: {:?}", trigger.target());

    commands
        .entity(trigger.target())
        .insert(Name::new("Client"))
        .insert(ReplicationReceiver::default())
        .insert(ReplicationSender::new(
            SERVER_REPLICATION_INTERVAL,
            SendUpdatesMode::SinceLastAck,
            false,
        ));
}

fn on_new_connection(
    trigger: Trigger<OnAdd, Connected>,
    mut commands: Commands,
    q_connected: Query<&RemoteId, With<ClientOf>>,
    _: Single<&Server>,
) -> Result {
    info!("New connection established: {:?}", trigger.target());

    debug!("Spawning character for {:?}", trigger.target());

    let entity = trigger.target();
    let RemoteId(peer) = q_connected.get(entity)?;

    commands.spawn((
        PlayerId(*peer),
        Name::new("Player"),
        ActionState::<NetworkCharacterAction>::default(),
        Position(Vec3::new(0.0, 3.0, 0.0)),
        Rotation::default(),
        Replicate::to_clients(NetworkTarget::All),
        PredictionTarget::to_clients(NetworkTarget::All),
        ControlledBy {
            owner: entity,
            lifetime: Lifetime::default(),
        },
        NetworkPlayerController,
        PhysicsCharacterBundle::default(),
        PhysicsCharacterInput::default(),
    ));

    Ok(())
}
