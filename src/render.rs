use bevy::app::{stage, AppBuilder, Plugin};
use bevy::asset::{AssetServer, Assets, Handle, HandleUntyped, LoadState};
use bevy::ecs::{IntoSystem, Res, ResMut, ShouldRun, SystemStage};
use bevy::render::texture::Texture;

use bevy::sprite::{TextureAtlas, TextureAtlasBuilder};

use bevy::utils::HashMap;

use std::path::Path;

const TEXTURES_PATH: &str = "textures";

pub(crate) struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<Textures>()
            .add_startup_system(render_setup.system())
            .add_stage_before(
                stage::POST_UPDATE,
                "create_sprites",
                SystemStage::parallel()
                    .with_system(create_texture_atlas.system())
                    .with_run_criteria(only_until_ready.system()),
            );
    }
}

#[derive(Default)]
pub(crate) struct Textures {
    handles: Option<Vec<HandleUntyped>>,
    lookup_table: HashMap<String, u32>,
    pub(crate) texture_atlas: Handle<TextureAtlas>,
}

impl Textures {
    #[inline]
    pub(crate) fn is_ready(&self) -> bool {
        self.handles.is_none()
    }
}

fn only_until_ready(value: Res<'_, Textures>) -> ShouldRun {
    if value.is_ready() {
        ShouldRun::No
    } else {
        ShouldRun::Yes
    }
}

fn render_setup(mut textures: ResMut<'_, Textures>, asset_server: Res<'_, AssetServer>) {
    let path: &Path = TEXTURES_PATH.as_ref();
    textures.handles = Some(asset_server.load_folder(path).unwrap());
}

fn create_texture_atlas(
    mut textures: ResMut<'_, Textures>,
    asset_server: Res<'_, AssetServer>,
    mut texture_atlases: ResMut<'_, Assets<TextureAtlas>>,
    mut texture_assets: ResMut<'_, Assets<Texture>>,
) {
    let textures = &mut *textures;
    match textures.handles.as_mut() {
        None => {}
        Some(handles) => {
            if let LoadState::Loaded =
                asset_server.get_group_load_state(handles.iter().map(|h| h.id))
            {
                let handles = textures.handles.take().unwrap();
                let handles: Vec<_> = handles.into_iter().map(|h| h.typed()).collect();
                // Create a texture atlas from the map textures
                let mut texture_atlas_builder = TextureAtlasBuilder::default();
                for handle in handles.iter() {
                    let texture = texture_assets.get(handle).unwrap();
                    texture_atlas_builder.add_texture(handle.as_weak(), &texture);
                }
                let texture_atlas = texture_atlas_builder.finish(&mut texture_assets).unwrap();

                // Associate the texture atlas handles with the corresponding texture name
                for handle in handles {
                    let texture_index = texture_atlas.get_texture_index(&handle).unwrap();
                    let texture_path = asset_server.get_handle_path(&handle).unwrap();
                    textures.lookup_table.insert(
                        texture_path
                            .path()
                            .file_stem()
                            .unwrap()
                            .to_string_lossy()
                            .into_owned(),
                        texture_index as u32,
                    );
                    texture_assets.remove(handle);
                }
                textures.texture_atlas = texture_atlases.add(texture_atlas);
            }
        }
    }
}
