use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    log::LogPlugin,
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap, OpaqueRendererMethod},
    prelude::*,
    tasks::{TaskPool, TaskPoolBuilder},
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_eventwork::{
    AppNetworkMessage, EventworkRuntime, Network, NetworkData, NetworkEvent, NetworkMessage,
};
use bevy_eventwork_mod_websockets::{NetworkSettings, WebSocketProvider};
use bevy_http_client::prelude::*;
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use config_loader::SmartHomeConfigAsset;
use device_model::DeviceModel;
use ehttp::{Headers, Request};
use errors::DeviceModelError;
use events::{LightModification, SceneModificationEvent, SunModification};
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

pub mod config;
pub mod config_loader;
pub mod device_model;
pub mod emoji;
pub mod errors;
pub mod events;
pub mod item;
pub mod openhab;
pub mod plot;
pub mod ui;
pub mod utils;
pub mod widget_settings;

use crate::{openhab::OpenHabState, utils::scale_value};
use std::f32::consts::*;

impl NetworkMessage for OpenHabState {
    const NAME: &'static str = "OpenHab"; // Needs to be identical with what's set in the websocket implementation (bevy_eventwork_mod_websockets)
}

const OPENHAB_IP: &str = "192.168.178.20";

const DEFAULT_LIGHT_INTENSITY: f32 = 50000.;
const DEFAULT_ENV_INTENSITY: f32 = 500.;
const CEILING_NAME: &str = "Plane.010";

#[derive(Component)]

struct EnergyFlow;

#[derive(Component)]
struct TestObject;

#[derive(Default, Resource)]
struct UiState {
    num_updates: i32,
    connection_status: String,
    sun: Option<Entity>,
    // Set to true once the material of the ceiling has been replaced.
    replaced_ceiling_material: bool,
    // Config
    config: Handle<SmartHomeConfigAsset>,
    // Sun temp
    elevation: f32,
    azimuth: f32,
}

fn main() {
    // Allow more HTTP clients, as each concurrent requests seems to be it's own client.
    // At boot up, we need one client per item to check for the state.
    let mut http_client_settings = HttpClientSetting::default();
    http_client_settings.client_limits = 1000;

    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: DEFAULT_ENV_INTENSITY,
        })
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .insert_resource(http_client_settings)
        .init_resource::<UiState>()
        .init_resource::<DeviceModel>()
        .add_event::<LightModification>()
        .add_event::<SunModification>()
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                filter: "info,bevy_eventwork=debug,bevy_eventwork_mod_websockets=debug".into(),
                level: bevy::log::Level::DEBUG,
                update_subscriber: None,
            }),
            FrameTimeDiagnosticsPlugin,
            EguiPlugin,
            bevy_eventwork::EventworkPlugin::<WebSocketProvider, bevy::tasks::TaskPool>::default(),
            // WorldInspectorPlugin::new(),
            OrbitCameraPlugin::default(),
            LookTransformPlugin,
            HttpClientPlugin,
        ))
        .insert_resource(NetworkSettings::default())
        // Task pool for network processing
        .insert_resource(EventworkRuntime(
            TaskPoolBuilder::new().num_threads(2).build(),
        ))
        // Add additional assets
        .init_asset::<config_loader::SmartHomeConfigAsset>()
        .init_asset_loader::<config_loader::SmartHomeConfigAssetLoader>()
        // Systems that create Egui widgets should be run during the `CoreSet::Update` set,
        // or after the `EguiSet::BeginFrame` system (which belongs to the `CoreSet::PreUpdate` set).
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                animate_sun,
                animate_lights,
                animate_objects,
                animate_paths,
                ui_example_system,
                handle_network_events,
                handle_connect,
                handle_state_change,
                handle_state_query_response,
                setup_config,
            ),
        )
        .listen_for_message::<OpenHabState, WebSocketProvider>()
        .run();
}

fn setup(
    mut ui_state: ResMut<UiState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut contexts: EguiContexts,
    asset_server: Res<AssetServer>,
) {
    let ctx: &egui::Context = contexts.ctx_mut();
    egui_extras::install_image_loaders(ctx);

    commands
        .spawn(Camera3dBundle::default())
        .insert(OrbitCameraBundle::new(
            OrbitCameraController::default(),
            Vec3::new(0., 20.0, 0.0),
            Vec3::new(0., 2., 0.),
            Vec3::Y,
        ));

    ui_state.sun = Some(
        commands
            .spawn(DirectionalLightBundle {
                directional_light: DirectionalLight {
                    illuminance: light_consts::lux::OVERCAST_DAY,
                    shadows_enabled: true,
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(0.0, 2.0, 0.0),
                    ..default()
                },
                // The default cascade config is designed to handle large scenes.
                // As this example has a much smaller world, we can tighten the shadow
                // bounds for better visual quality.
                cascade_shadow_config: CascadeShadowConfigBuilder {
                    first_cascade_far_bound: 4.0, // was 4
                    maximum_distance: 100.0,      // was 100
                    ..default()
                }
                .into(),
                ..default()
            })
            .id(),
    );

    commands.spawn(SceneBundle {
        scene: asset_server.load("sihlterrassen.gltf#Scene0"),
        ..default()
    });

    ui_state.config = asset_server.load("items.json");

    let energy_flow_material = materials.add(StandardMaterial {
        base_color: Color::RED,
        emissive: Color::RED,
        reflectance: 0.,
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::default().mesh().ico(3).unwrap()),
            material: energy_flow_material.clone(),
            transform: Transform::from_xyz(0.0, 2.0, -2.6).with_scale(Vec3::splat(0.1)),
            ..default()
        },
        EnergyFlow,
    ));
}

fn handle_connect(
    mut ui_state: ResMut<UiState>,
    net: ResMut<Network<WebSocketProvider>>,
    settings: Res<NetworkSettings>,
    task_pool: Res<EventworkRuntime<TaskPool>>,
) {
    if ui_state.connection_status == "" {
        ui_state.connection_status = "Connecting".to_string();
        // Connect Websocket for Smart Home updates
        net.connect(
            url::Url::parse(&format!(
                "ws://{}:8080/ws?topic=smarthome/items/*/*",
                OPENHAB_IP
            ))
            .unwrap(),
            &task_pool.0,
            &settings,
        );
    }
}

/// Receive HTTP replies from calls made using the HTTP client.
///
/// Those include state requests as well as commands sent requesting for item changes.
fn handle_state_query_response(
    mut device_model: ResMut<DeviceModel>,
    mut ev_resp: EventReader<HttpResponse>,
    mut ev_light_modification: EventWriter<LightModification>,
    mut ev_sun_modification: EventWriter<SunModification>,
) {
    for response in ev_resp.read() {
        // Format of the URL: http://192.168.178.20:8080/rest/items/OutdoorTemperature/state
        // Another format:    http://192.168.178.20:8080/rest/items/zimmer_2_steckdose
        info!("Received HTTP response for state request: {:?}", response);
        let new_state = String::from_utf8(response.bytes.clone());
        if let Some(item) = response.url.strip_suffix("/state") {
            if let Some(item) = item.split("/").last() {
                if let Ok(new_state) = new_state {
                    info!("Received state query response: {} <- {}", item, new_state);
                    match device_model.state_changed(item, &new_state) {
                        Ok(scene_modifications) => register_scene_modifications(
                            &mut ev_light_modification,
                            &mut ev_sun_modification,
                            scene_modifications,
                        ),
                        Err(e) => error!("Handling state query response failed: {:?}", e),
                    }
                }
            }
        }
    }
}

fn animate_sun(
    ui_state: ResMut<UiState>,
    mut sun_events: EventReader<SunModification>,
    mut query: Query<(Entity, &mut DirectionalLight, &mut Transform), With<DirectionalLight>>,
    mut ambient_light: ResMut<AmbientLight>,
) {
    for sun_event in sun_events.read() {
        for (entity, mut light, mut transform) in &mut query {
            if Some(entity) == ui_state.sun {
                fn degree_to_radians(val: f32) -> f32 {
                    val * PI / 180.
                }

                let azimuth = sun_event.azimuth;
                let elevation = sun_event.elevation;

                info!("Setting sun to be: {:?}", sun_event);

                // Given the azimuth and elevation of the sun, you can calculate the Euler angles as follows:

                // First Euler Angle (Yaw): Set the first Euler angle (yaw) to the azimuth of the sun.
                // Second Euler Angle (Pitch): Set the second Euler angle (pitch) to the elevation of the sun.

                // To apply these rotations, you would first rotate around the y-axis by the yaw angle,
                // then around the x-axis by the pitch angle. This will align the directional light with the sun's direction.

                // Remember that the order of rotations is important. In this case, the yaw (azimuth) rotation should
                // be applied first, followed by the pitch (elevation) rotation.

                // YXZ can be used for yaw (y-axis), pitch (x-axis), roll (z-axis).
                transform.rotation = Quat::from_euler(
                    EulerRot::YXZ,
                    degree_to_radians(-1.0 * (azimuth - 180.)), // y == yaw
                    degree_to_radians(-1.0 * elevation),        // x == pitch
                    0.,
                );

                // Map 0 lux to 0 and 30000 lux to 15
                let default_sun_mapping = vec![(0., 0.), (30000., 15000.)];
                let sun_energy = scale_value(sun_event.illuminance as f32, &default_sun_mapping);

                light.illuminance = sun_energy;
            }

            // Set intensitiv of environmental light
            let default_ambient_mapping = vec![(0., 200.), (10000., 1700.)];
            ambient_light.brightness =
                scale_value(sun_event.illuminance as f32, &default_ambient_mapping);
        }
    }
}

fn animate_paths(mut query: Query<&mut Transform, With<EnergyFlow>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.translation = Vec3::new(1.0 * (time.elapsed_seconds() % 5.0), 3.0, 0.0);
    }
}

fn animate_lights(
    mut point_light: Query<(&Name, &mut PointLight)>,
    mut spot_light: Query<(&Name, &mut SpotLight)>,
    mut light_events: EventReader<LightModification>,
) {
    for event in light_events.read() {
        debug!("Light modification event: {:?}", event);
        let mut found = false;
        // Point lights
        for (name, mut light) in &mut point_light {
            if name.as_str() == &event.entity_name {
                light.intensity = event.illuminance_percentage * DEFAULT_LIGHT_INTENSITY;
                found = true;
            }
        }
        // Spot lights
        for (name, mut light) in &mut spot_light {
            if name.as_str() == &event.entity_name {
                light.intensity = event.illuminance_percentage * DEFAULT_LIGHT_INTENSITY;
                found = true;
            }
        }

        if !found {
            info!(
                "Could not find light with name {} for event {:?}",
                event.entity_name, event
            );
        } else {
            debug!("Successfully executed {} to {:?}", event.entity_name, event);
        }
    }
}

// https://github.com/bevyengine/bevy/blob/main/examples/3d/transparency_3d.rs
// https://github.com/bevyengine/bevy/discussions/6907
// https://github.com/mvlabat/muddle-run/blob/b2fc5d91a6b6d81ee94029924507db0acdedc42d/bins/desktop_client/src/main.rs
fn animate_objects(
    mut ui_state: ResMut<UiState>,
    device_model: Res<DeviceModel>,
    mut query: Query<(&Name, &mut Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ev_request: EventWriter<HttpRequest>,
) {
    if !ui_state.replaced_ceiling_material {
        for (name, material) in query.iter_mut() {
            if name.as_str() == CEILING_NAME {
                println!(
                    "Replacing material of ceiling to be invisible: {} - material: {}",
                    name,
                    material.id()
                );

                ui_state.azimuth = 320.;
                ui_state.elevation = 57.;

                // Determined using bevy_inspector_egui
                if let Some(m) = materials.get_mut(material.id()) {
                    m.alpha_mode = AlphaMode::Opaque;
                    m.opaque_render_method = OpaqueRendererMethod::Deferred;
                }
                ui_state.replaced_ceiling_material = true;

                // Get a list of all  items and fetch the initial state for all of them.
                for item in device_model.get_items() {
                    info!("Requesting state of item: {}", item);
                    let request = HttpClient::new()
                        .get(&format!(
                            "http://{}:8080/rest/items/{}/state",
                            OPENHAB_IP, item
                        ))
                        .build();
                    ev_request.send(request);
                }
            }
        }
    }
}

fn ui_example_system(
    ui_state: Res<UiState>,
    mut device_model: ResMut<DeviceModel>,
    diagnostics: Res<DiagnosticsStore>,
    mut contexts: EguiContexts,
    camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut ev_request: EventWriter<HttpRequest>,
) {
    // Determine FPS from diagnostics data
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .map(|s| s.value())
        .flatten()
        .unwrap_or(-1.);

    egui::Window::new("Debug").show(contexts.ctx_mut(), |ui| {
        ui.label(format!("fps: {fps:.2}"));
        ui.label(format!("Num views: {}", device_model.views.len()));
        ui.label(format!("Num widgets: {}", device_model.widgets.len()));
        ui.label(format!("Current view: {:?}", device_model.current_view));
        ui.label(format!("Fullscreen: {:?}", device_model.fullscreen_widget));
        if let Some(current_view) = &device_model.current_view {
            ui.label(format!(
                "Num current widgets: {:?}",
                device_model.views.get(current_view).map(|e| e.len())
            ));
        }
        ui.label(ui_state.connection_status.clone());
        ui.label(format!("Num updates {}", ui_state.num_updates));
    });

    // Get camera for viewport position calculations.
    let (camera, camera_global_transform) = camera.single();

    // Render widgets and forward HTTP call events generated by widgets
    match device_model.render(contexts.ctx_mut(), camera, camera_global_transform) {
        Ok(requested_state_changes) => {
            for requested_state_change in requested_state_changes {
                let request = Request {
                    method: "POST".to_string(),
                    url: format!(
                        "http://{}:8080/rest/items/{}",
                        OPENHAB_IP, requested_state_change.item
                    ),
                    body: requested_state_change.value.as_bytes().to_vec(),
                    headers: Headers::new(&[("Content-Type", "text/plain")]),
                    #[cfg(target_arch = "wasm32")]
                    mode: ehttp::Mode::Cors,
                };

                ev_request.send(HttpClient::new().request(request).build());
            }
        }
        Err(e) => error!("Failed to render current view: {:?}", e),
    }
}

fn handle_network_events(
    mut new_network_events: EventReader<NetworkEvent>,
    mut ui_state: ResMut<UiState>,
) {
    for event in new_network_events.read() {
        info!("Received event");
        match event {
            NetworkEvent::Connected(_) => {
                info!("Succesfully connected to server!");
                ui_state.connection_status = "Connected".to_string();
            }

            NetworkEvent::Disconnected(_) => {
                info!("Disconnected from server!");
                ui_state.connection_status = "Disconnected".to_string();
            }
            NetworkEvent::Error(err) => {
                info!("Server connection failed: {}", err);
                ui_state.connection_status = format!("Error: {}", err);
            }
        }
    }
}

/// Registers the given scene modifications by sending them as events.
fn register_scene_modifications(
    ev_light_modification: &mut EventWriter<LightModification>,
    ev_sun_modification: &mut EventWriter<SunModification>,
    modifications: Vec<SceneModificationEvent>,
) {
    for event in modifications {
        match event {
            SceneModificationEvent::LightModification(ev) => {
                ev_light_modification.send(ev);
            }
            SceneModificationEvent::SunModification(ev) => {
                ev_sun_modification.send(ev);
            }
            _ => error!("Unsupported scene modification: {:?}", event),
        }
    }
}

/// Handle Messages coming from the backend's websocket.
///
/// Those are a sequence of state changes of the backend's items. We pass those to the device model
/// for forwarding them to corresponding widgets.
///
/// In response, we get a list of requested 3d scene changes.
fn handle_state_change(
    mut new_messages: EventReader<NetworkData<OpenHabState>>,
    mut ui_state: ResMut<UiState>,
    mut ev_light_modification: EventWriter<LightModification>,
    mut ev_sun_modification: EventWriter<SunModification>,
    mut device_model: ResMut<DeviceModel>,
) {
    for new_message in new_messages.read() {
        ui_state.num_updates += 1;
        match device_model.backend_state_changed(&new_message.topic, &new_message.payload) {
            Ok(device_modifications) => register_scene_modifications(
                &mut ev_light_modification,
                &mut ev_sun_modification,
                device_modifications,
            ),
            Err(DeviceModelError::ItemNotFound(e)) => {
                debug!("Received update for unknown item: {:?}", e)
            }
            Err(e) => error!("Handling item state change failed: {:?}", e),
        };
    }
}

fn setup_config(
    mut device_model: ResMut<DeviceModel>,
    mut ev_request: EventWriter<HttpRequest>,
    ui_state: Res<UiState>,
    config: Res<Assets<config_loader::SmartHomeConfigAsset>>,
) {
    if !device_model.initialized {
        let config = config.get(&ui_state.config).clone();
        if let Some(config) = config {
            device_model.parse(&config.config);
        }
    }
}
