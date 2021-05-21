use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use ndarray::prelude::*;
use std::ops::{Deref, DerefMut};

use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::utils::BoxedFuture;
use ca_formats::rle::Rle;

#[derive(Default)]
pub struct CafLoader;

impl AssetLoader for CafLoader {
    fn extensions(&self) -> &[&str] {
        &["rle"]
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let loaded = Rle::new(std::str::from_utf8(&bytes).unwrap())?;
            let mut board = Board::new(
                loaded.header_data().unwrap().x as usize,
                loaded.header_data().unwrap().y as usize,
            );
            for cell in loaded.into_iter() {
                let pos = cell.unwrap().position;
                if let Some(c) = board.get_mut((pos.0 as usize, pos.1 as usize)) {
                    *c = 1u8;
                }
            }
            // let board = Board::new(512, 512);
            dbg!("board created out of thin air");
            load_context.set_default_asset(LoadedAsset::new(board));
            Ok(())
        })
    }
}

#[derive(Debug, TypeUuid, Clone)]
#[uuid = "3e6c203c-76a0-4acc-a812-8d48ee685e61"]
pub struct Board(pub Array2<u8>);

impl Board {
    pub fn new(width: usize, height: usize) -> Board {
        Board(Array2::zeros((width, height)))
    }
}

impl Deref for Board {
    type Target = Array2<u8>;
    fn deref(&self) -> &Array2<u8> {
        &self.0
    }
}

impl DerefMut for Board {
    fn deref_mut(&mut self) -> &mut Array2<u8> {
        &mut self.0
    }
}

#[derive(Default)]
struct BoardState {
    handle: Handle<Board>,
    loaded: bool,
}

#[derive(Clone)]
struct Cell(bool);

pub struct GameOfLife {
    cells: Board,
    scratch: Board,
    texture: Handle<Texture>,
}

impl GameOfLife {
    fn new(width: usize, height: usize, texture: Handle<Texture>) -> Self {
        GameOfLife {
            cells: Board::new(width + 2, height + 2),
            scratch: Board::new(width, height),
            texture,
        }
    }

    fn new_from_board(board: &Board, texture: Handle<Texture>) -> Self {
        let width = board.nrows();
        let height = board.ncols();
        GameOfLife {
            cells: board.clone(),
            scratch: Board::new(width - 2, height - 2),
            texture,
        }
    }

    // from https://github.com/rust-ndarray/ndarray/blob/master/examples/life.rs

    fn iterate(&mut self) {
        let z = &mut self.cells;
        // compute number of neighbors
        let mut neigh = self.scratch.view_mut();
        neigh.fill(0);
        neigh += &z.slice(s![0..-2, 0..-2]);
        neigh += &z.slice(s![0..-2, 1..-1]);
        neigh += &z.slice(s![0..-2, 2..]);

        neigh += &z.slice(s![1..-1, 0..-2]);
        neigh += &z.slice(s![1..-1, 2..]);

        neigh += &z.slice(s![2.., 0..-2]);
        neigh += &z.slice(s![2.., 1..-1]);
        neigh += &z.slice(s![2.., 2..]);

        // birth where n = 3 and z[i] = 0,
        // survive where n = 2 || n = 3 and z[i] = 1
        let mut zv = z.slice_mut(s![1..-1, 1..-1]);

        // this is autovectorized amazingly well!
        zv.zip_mut_with(&neigh, |y, &n| *y = ((n == 3) || (n == 2 && *y > 0)) as u8);
    }
}

fn main() {
    let mut app = App::build();

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(DefaultPlugins);

    #[cfg(target_arch = "wasm32")]
    app.add_plugins(bevy_webgl2::DefaultPlugins);

    app.add_asset::<Board>().init_asset_loader::<CafLoader>();
    app.init_resource::<BoardState>();

    app.add_startup_system(setup.system())
        .add_startup_system(load_board.system())
        .add_system(grid_system.system())
        .add_system(setup_board.system())
        .run();
}

fn load_board(mut state: ResMut<BoardState>, asset_server: Res<AssetServer>) {
    state.handle = asset_server.load("patterns/queenbeeloop.rle");
    dbg!("loading board");
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    dbg!("setup camera");
}

fn setup_board(
    mut commands: Commands,
    mut state: ResMut<BoardState>,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    boards: Res<Assets<Board>>,
) {
    let board = boards.get(&state.handle);
    if state.loaded || board.is_none() {
        return;
    };
    dbg!("setting up the board");

    let border = 6;

    let a = board.unwrap();
    let a = {
        let mut b = Board::new(a.nrows() + border, a.ncols() + border);
        let border = (border / 2) as i32;
        b.slice_mut(s![border..-border, border..-border])
            .assign(&a.0);
        b
    };

    let width = a.nrows();
    let height = a.ncols();

    let texture = textures.add(Texture::new_fill(
        bevy::render::texture::Extent3d::new(width as u32, height as u32, 1u32),
        bevy::render::texture::TextureDimension::D2,
        &[0u8, 0u8, 0u8, 255u8],
        bevy::render::texture::TextureFormat::Rgba8UnormSrgb,
    ));

    let material = ColorMaterial::texture(texture.clone());
    //
    let scale = 8.0;

    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(material),
            sprite: Sprite {
                size: Vec2::new(width as f32, height as f32),
                ..Default::default()
            },
            transform: Transform::from_scale(Vec3::new(scale, scale, 1.0)),
            ..Default::default()
        })
        .insert(GameOfLife::new_from_board(&a, texture));

    state.loaded = true;
}

fn grid_system(mut grid_query: Query<&mut GameOfLife>, mut textures: ResMut<Assets<Texture>>) {
    for mut grid in grid_query.iter_mut() {
        grid.iterate();

        if let Some(texture) = textures.get_mut(&grid.texture) {
            texture.data = grid
                .cells
                .iter()
                .flat_map(|cell| {
                    if *cell > 0 {
                        [0u8, 0u8, 0u8, 255u8]
                    } else {
                        [255u8, 255u8, 255u8, 255u8]
                    }
                })
                .collect();
        }
    }
}
