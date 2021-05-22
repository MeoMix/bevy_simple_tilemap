use bevy::{
    prelude::*,
    render::{pipeline::PipelineDescriptor, render_graph::RenderGraph},
};

#[derive(Default)]
pub struct SimpleTileMapPlugin;

#[derive(Clone, Debug, Eq, Hash, PartialEq, StageLabel)]
pub enum SimpleTileMapStage {
    Update,
    Remesh,
}

impl Plugin for SimpleTileMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_stage_before(
            CoreStage::PostUpdate,
            SimpleTileMapStage::Update,
            SystemStage::parallel(),
        )
        .add_stage_after(
            SimpleTileMapStage::Update,
            SimpleTileMapStage::Remesh,
            SystemStage::parallel(),
        )
        .add_system_to_stage(SimpleTileMapStage::Update, crate::tilemap::update_chunk_system.system())
        .add_system_to_stage(
            SimpleTileMapStage::Remesh,
            crate::tilemap::update_chunk_mesh_system.system(),
        );

        let world = app.world_mut();

        let world_cell = world.cell();
        let mut render_graph = world_cell.get_resource_mut::<RenderGraph>().unwrap();
        let mut pipelines = world_cell.get_resource_mut::<Assets<PipelineDescriptor>>().unwrap();
        let mut shaders = world_cell.get_resource_mut::<Assets<Shader>>().unwrap();

        crate::render::add_tilemap_graph(&mut render_graph, &mut pipelines, &mut shaders);
    }
}
