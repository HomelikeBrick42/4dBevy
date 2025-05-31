use bevy::{
    app::{App, MainScheduleOrder, Plugin, PostUpdate},
    ecs::{
        entity::Entity,
        event::EventReader,
        query::With,
        resource::Resource,
        schedule::ScheduleLabel,
        system::{Res, ResMut},
    },
    log::info,
    window::{PrimaryWindow, RawHandleWrapperHolder, WindowResized},
};
use std::{cell::Cell, rc::Rc};

#[derive(ScheduleLabel, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PreRender;
#[derive(ScheduleLabel, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct StartRender;
#[derive(ScheduleLabel, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Render;
#[derive(ScheduleLabel, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Present;

struct RenderInitState {
    state: Rc<Cell<Option<RenderState>>>,
}

#[derive(Resource)]
pub struct RenderState {
    pub instance: wgpu::Instance,
    pub primary_window_entity: Entity,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

#[derive(Resource, Default)]
pub struct Rendering {
    pub surface_texture: Option<wgpu::SurfaceTexture>,
    pub command_buffers: Vec<wgpu::CommandBuffer>,
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        let render_init_state = RenderInitState {
            state: Rc::new(Cell::new(None)),
        };
        let render_init_future = {
            let state = render_init_state.state.clone();

            let (primary_window_entity, primary_window) = app
                .world_mut()
                .query_filtered::<(Entity, &RawHandleWrapperHolder), With<PrimaryWindow>>()
                .single(app.world())
                .expect("there should be a primary window");
            let primary_window = primary_window.clone();

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

                let surface_config = wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    width: 1,
                    height: 1,
                    present_mode: wgpu::PresentMode::AutoVsync,
                    desired_maximum_frame_latency: 2,
                    alpha_mode: wgpu::CompositeAlphaMode::Auto,
                    view_formats: vec![],
                };
                surface.configure(&device, &surface_config);

                info!("Initialized renderer");

                state.set(Some(RenderState {
                    primary_window_entity,
                    instance,
                    surface,
                    surface_config,
                    device,
                    queue,
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
        main_schedule.insert_after(PreRender, StartRender);
        main_schedule.insert_after(StartRender, Render);
        main_schedule.insert_after(Render, Present);

        app.init_resource::<Rendering>()
            .add_systems(StartRender, start_render)
            .add_systems(Present, present);
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

fn start_render(
    mut state: ResMut<RenderState>,
    mut resize_events: EventReader<WindowResized>,
    mut rendering: ResMut<Rendering>,
) -> bevy::ecs::error::Result {
    let RenderState {
        instance: _,
        primary_window_entity,
        ref surface,
        ref mut surface_config,
        ref device,
        queue: _,
    } = *state;

    if let Some(resize_event) = resize_events
        .read()
        .filter(|event| event.window == primary_window_entity)
        .last()
    {
        surface_config.width = resize_event.width as _;
        surface_config.height = resize_event.height as _;
        surface.configure(device, surface_config);

        info!(
            "Resized surface to {}, {}",
            surface_config.width, surface_config.height
        );
    }

    let surface_texture = match surface.get_current_texture() {
        Ok(texture) => texture,
        Err(wgpu::SurfaceError::Outdated) => return Ok(()),
        Err(wgpu::SurfaceError::Timeout) => return Ok(()),
        r => r?,
    };

    rendering.surface_texture = Some(surface_texture);

    Ok(())
}

fn present(state: Res<RenderState>, mut rendering: ResMut<Rendering>) {
    state
        .queue
        .submit(std::mem::take(&mut rendering.command_buffers));
    if let Some(surface_texture) = rendering.surface_texture.take() {
        surface_texture.present();
    }
}
