use bevy::{
    app::{App, MainScheduleOrder, Plugin, PostUpdate},
    ecs::{query::With, resource::Resource, schedule::ScheduleLabel, system::ResMut},
    log::info,
    window::{PrimaryWindow, RawHandleWrapperHolder},
};
use camera::GpuCamera;
use std::{cell::Cell, rc::Rc};

mod camera;

pub use camera::{Camera, MainCamera};

#[derive(ScheduleLabel, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PreRender;

#[derive(ScheduleLabel, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Render;

struct RenderInitState {
    state: Rc<Cell<Option<RenderState>>>,
}

#[derive(Resource)]
pub struct RenderState {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    camera_buffer: wgpu::Buffer,
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        let render_init_state = RenderInitState {
            state: Rc::new(Cell::new(None)),
        };
        let render_init_future = {
            let state = render_init_state.state.clone();

            let primary_window = app
                .world_mut()
                .query_filtered::<&RawHandleWrapperHolder, With<PrimaryWindow>>()
                .single(app.world())
                .expect("there should be a primary window")
                .clone();

            async move {
                let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
                    backends: wgpu::Backends::all(),
                    flags: wgpu::InstanceFlags::from_env_or_default(),
                    backend_options: wgpu::BackendOptions::from_env_or_default(),
                });

                let surface = {
                    // this is horrible but just do an async spin loop until the primary window is available, this seems to work
                    let handle = loop {
                        let handle = primary_window.0.lock().unwrap();
                        if handle.is_some() {
                            break handle;
                        }
                        drop(handle);
                        bevy::tasks::futures_lite::future::yield_now().await;
                    };
                    let handle = handle.as_ref().unwrap();

                    // Safety: this async task is spawned on the main thread with spawn_local, and bevy should only be initialised on the main thread
                    let handle = unsafe { handle.get_handle() };

                    instance
                        .create_surface(handle)
                        .expect("wgpu surface should be created")
                };

                info!("Created the surface");

                let adapter = instance
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::HighPerformance,
                        force_fallback_adapter: false,
                        compatible_surface: Some(&surface),
                    })
                    .await
                    .expect("wgpu adapter should be found");

                info!("Using adapter {:?}", adapter.get_info());

                let (device, queue) = adapter
                    .request_device(&wgpu::DeviceDescriptor {
                        label: Some("Device"),
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::default(),
                        memory_hints: wgpu::MemoryHints::Performance,
                        trace: wgpu::Trace::Off,
                    })
                    .await
                    .expect("wgpu device and queue should be created");

                info!("Created device and queue");

                let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Camera Buffer"),
                    size: size_of::<GpuCamera>() as _,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                    mapped_at_creation: false,
                });

                state.set(Some(RenderState {
                    instance,
                    surface,
                    device,
                    queue,
                    camera_buffer,
                }));
            }
        };
        bevy::tasks::IoTaskPool::get()
            .spawn_local(render_init_future)
            .detach();

        app.insert_non_send_resource(render_init_state);

        app.init_schedule(PreRender).init_schedule(Render);
        let mut main_schedule = app.world_mut().resource_mut::<MainScheduleOrder>();
        main_schedule.insert_after(PostUpdate, PreRender);
        main_schedule.insert_after(PreRender, Render);

        app.register_type::<Camera>()
            .register_type::<MainCamera>()
            .add_systems(PreRender, camera::upload_camera)
            .add_systems(Render, render);
    }

    fn ready(&self, app: &App) -> bool {
        let init_resource = app.world().non_send_resource::<RenderInitState>();
        let init_state = init_resource.state.take();
        let finished = init_state.is_some();
        init_resource.state.set(init_state);
        finished
    }

    fn finish(&self, app: &mut App) {
        let render_state = app
            .world_mut()
            .remove_non_send_resource::<RenderInitState>()
            .unwrap()
            .state
            .take()
            .expect("if RenderPlugin::ready returned true then RenderState has been created");
        app.insert_resource(render_state);
    }
}

fn render(state: ResMut<RenderState>) {
    _ = state;
}
