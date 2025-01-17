use crate::MandelbulbUniforms;
use bevy::{
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::{
        render_resource::{encase, AsBindGroup, OwnedBindingResource, ShaderRef, ShaderType},
        renderer::RenderQueue,
        Extract, Render, RenderApp, RenderSet,
    },
    sprite::{Material2d, Material2dPlugin, RenderMaterials2d},
};

use crate::AspectRatio;

pub struct RayMarchingMaterialPlugin;

impl Plugin for RayMarchingMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<RayMarchingMaterial>::default());

        //Add our custom extract and prepare systems to the app
        app.sub_app_mut(RenderApp)
            .add_systems(ExtractSchedule, extract_raymarching_material)
            .add_systems(
                Render,
                prepare_raymarching_material.in_set(RenderSet::Prepare),
            );
    }
}

//New material created to setup custom shader
#[derive(AsBindGroup, Debug, Clone, TypeUuid, TypePath)]
#[uuid = "084f230a-b958-4fc4-8aaf-ca4d4eb16412"]
pub struct RayMarchingMaterial {
    //Set the uniform at binding 0 to have the following information - connects to Camera struct in ray_marching_material.wgsl
    #[uniform(0)]
    pub camera_position: Vec3,
    #[uniform(0)]
    pub camera_forward: Vec3,
    #[uniform(0)]
    pub camera_horizontal: Vec3,
    #[uniform(0)]
    pub camera_vertical: Vec3,
    #[uniform(0)]
    pub aspect_ratio: f32,
    #[uniform(0)]
    pub power: f32,
    #[uniform(0)]
    pub max_iterations: u32,
    #[uniform(0)]
    pub bailout: f32,
    #[uniform(0)]
    pub num_steps: u32,
    #[uniform(0)]
    pub min_dist: f32,
    #[uniform(0)]
    pub max_dist: f32,
    #[uniform(0)]
    pub zoom: f32,
}

impl RayMarchingMaterial {
    pub fn new() -> RayMarchingMaterial {
        RayMarchingMaterial {
            camera_position: Vec3::new(0.0, 0.0, 0.0),
            camera_forward: Vec3::new(0.0, 0.0, -1.0),
            camera_horizontal: Vec3::new(1.0, 0.0, 0.0),
            camera_vertical: Vec3::new(0.0, 1.0, 0.0),
            aspect_ratio: 1.0,
            power: 8.0,
            max_iterations: 8,
            bailout: 3.0,
            num_steps: 64,
            min_dist: 0.002,
            max_dist: 1000.0,
            zoom: 1.0,
        }
    }
}

//Setup the RayMarchingMaterial to use the custom shader file for the vertex and fragment shader
//Note: one of these can be removed to use the default material 2D bevy shaders for the vertex/fragment shader
impl Material2d for RayMarchingMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/ray_marching_material.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/ray_marching_material.wgsl".into()
    }
}

//Uniform data struct to move data from the "Game World" to the "Render World" with the ShaderType derived
#[derive(ShaderType, Clone)]
struct RayMarchingMaterialUniformData {
    camera_position: Vec3,
    camera_forward: Vec3,
    camera_horizontal: Vec3,
    camera_vertical: Vec3,
    apsect_ratio: f32,
    power: f32,
    max_iterations: u32,
    bailout: f32,
    num_steps: u32,
    min_dist: f32,
    max_dist: f32,
    zoom: f32,
}

//Move information from the "Game World" to the "Render World"
fn extract_raymarching_material(
    mut commands: Commands,
    ray_marching_query: Extract<Query<(Entity, &Handle<RayMarchingMaterial>)>>,
    aspect_ratio_resource: Extract<Res<AspectRatio>>,
    mandelbulb_uniform_resource: Extract<Res<MandelbulbUniforms>>,
    camera_query: Extract<Query<&Transform, With<Camera2d>>>,
) {
    for (entity, material_handle) in ray_marching_query.iter() {
        let mut entity = commands.get_or_spawn(entity);
        entity.insert(material_handle.clone());
        for transform in camera_query.iter() {
            entity.insert(*transform);
        }
    }

    commands.insert_resource(AspectRatio {
        aspect_ratio: aspect_ratio_resource.aspect_ratio,
    });
    commands.insert_resource(MandelbulbUniforms {
        power: mandelbulb_uniform_resource.power,
        max_iterations: mandelbulb_uniform_resource.max_iterations,
        bailout: mandelbulb_uniform_resource.bailout,
        num_steps: mandelbulb_uniform_resource.num_steps,
        min_dist: mandelbulb_uniform_resource.min_dist,
        max_dist: mandelbulb_uniform_resource.max_dist,
        zoom: mandelbulb_uniform_resource.zoom,
    });
}

//Update the buffers with the data taken from the "Game World" and sent to the "Render World" so they can be used by the GPU
fn prepare_raymarching_material(
    materials: Res<RenderMaterials2d<RayMarchingMaterial>>,
    material_query: Query<(&Transform, &Handle<RayMarchingMaterial>)>,
    render_queue: Res<RenderQueue>,
    aspect_ratio_resource: Res<AspectRatio>,
    mandelbulb_uniform_resource: Res<MandelbulbUniforms>,
) {
    for (transform, material_handle) in &material_query {
        if let Some(material) = materials.get(material_handle) {
            for binding in material.bindings.iter() {
                if let OwnedBindingResource::Buffer(current_buffer) = binding {
                    let mut buffer = encase::UniformBuffer::new(Vec::new());
                    buffer
                        .write(&RayMarchingMaterialUniformData {
                            camera_position: transform.translation,
                            camera_forward: transform.forward(),
                            camera_horizontal: transform.right(),
                            camera_vertical: transform.up(),
                            apsect_ratio: aspect_ratio_resource.aspect_ratio,
                            power: mandelbulb_uniform_resource.power,
                            max_iterations: mandelbulb_uniform_resource.max_iterations,
                            bailout: mandelbulb_uniform_resource.bailout,
                            num_steps: mandelbulb_uniform_resource.num_steps,
                            min_dist: mandelbulb_uniform_resource.min_dist,
                            max_dist: mandelbulb_uniform_resource.max_dist,
                            zoom: mandelbulb_uniform_resource.zoom,
                        })
                        .unwrap();
                    //Write to an offset in the buffer so the position data is not over-written
                    render_queue.write_buffer(&current_buffer, 0, buffer.as_ref());
                }
            }
        }
    }
}
