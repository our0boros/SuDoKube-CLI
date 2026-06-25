use crate::input::picker::PickerCamera;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct OrbitCamera {
    pub radius: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub target: Vec3,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            radius: 28.0,
            yaw: 0.7,
            pitch: 0.5,
            target: Vec3::ZERO,
        }
    }
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 28.0).looking_at(Vec3::ZERO, Vec3::Y),
        OrbitCamera::default(),
        PickerCamera,
    ));

    commands.spawn(AmbientLight {
        color: Color::WHITE,
        brightness: 0.6,
        affects_lightmapped_meshes: false,
    });

    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 2500.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(10.0, 15.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub fn update_camera(
    mut cameras: Query<(&mut Transform, &mut OrbitCamera)>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    let Ok((mut transform, mut camera)) = cameras.single_mut() else {
        return;
    };

    let mut delta = Vec2::ZERO;
    if mouse_buttons.pressed(MouseButton::Right) {
        delta += motion.delta;
    }

    let radius_delta = -scroll.delta.y;

    if delta.length_squared() > 0.0 || radius_delta != 0.0 {
        camera.yaw -= delta.x * 0.005;
        camera.pitch -= delta.y * 0.005;
        camera.pitch = camera.pitch.clamp(-1.4, 1.4);
        camera.radius = (camera.radius + radius_delta * 0.5).clamp(10.0, 60.0);

        let x = camera.radius * camera.pitch.cos() * camera.yaw.cos();
        let y = camera.radius * camera.pitch.sin();
        let z = camera.radius * camera.pitch.cos() * camera.yaw.sin();
        *transform = Transform::from_xyz(x, y, z).looking_at(camera.target, Vec3::Y);
    }
}
