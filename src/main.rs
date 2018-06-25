#![feature(crate_visibility_modifier, trace_macros, nll)]

extern crate cgmath;
extern crate gl;
extern crate glutin;
extern crate image;
extern crate specs;
extern crate rand;

#[macro_use]
mod gl_api;
mod grid;
mod tiles;

use tiles::Tile;
use image::GenericImage;
use gl_api::texture::Texture;
use gl_api::texture::*;
use rand::Rng;
use cgmath::Vector3;
use cgmath::{Vector2, Vector4};
use gl_api::buffer::{ShaderStorageBuffer, VertexBuffer};
use gl_api::uniform::Uniform;
use gl_api::vertex_array::VertexArray;
use glutin::GlContext;
use glutin::{Api, GlRequest};
use specs::shred::PanicHandler;
use std::marker::PhantomData;

use gl_api::buffer::UsageType;
use gl_api::shader::program::*;
use gl_api::shader::shader::*;

const MAP_WIDTH: usize = 40;
const MAP_HEIGHT: usize = 40;

macro_rules! newtype {
    (@DEREF $name:ident is $type:ty) => {
        impl ::std::ops::Deref for $name {
            type Target = $type;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl ::std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
    ($name:ident is $type:ty) => {
        pub struct $name($type);
        newtype!(@DEREF $name is $type);
    };
    (#[$($meta:meta),+] $name:ident is $type:ty) => {
        #[$($meta),+]
        pub struct $name($type);
        newtype!(@DEREF $name is $type);
    };
}

use grid::GridX;

newtype!(#[derive(Debug)] TileGrid is GridX<Entity>);
newtype!(#[derive(Debug)] TilePos is Vector2<usize>);
newtype!(#[derive(Debug)] Terrain is Vector2<usize>);
// newtype!(TerrainColor is Vector4<f32>);
newtype!(#[derive(Debug)] TerrainSprite is Tile);
newtype!(#[derive(Debug, Default)] ModifiedTerrain is (bool, BitSet));
newtype!(#[derive(Debug, Default)] NewTerrain is (bool, BitSet));

pub struct TerrainColor {
    fg: Vector4<f32>,
    bg: Vector4<f32>,
}

#[derive(Debug)]
pub struct ChangeSet(HashMap<Vector2<usize>, Entity>);

#[derive(Default)]
pub struct Dirty;

use specs::prelude::*;

impl Component for TilePos {
    type Storage = VecStorage<Self>;
}

impl Component for TerrainSprite {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

impl Component for TerrainColor {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

impl Component for Terrain {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

impl Component for Dirty {
    type Storage = NullStorage<Self>;
}

vertex! {
    vertex WorldVertex {
        pos: Vector2<f32>,
        color: Vector3<f32>,
    }
}

struct WorldUniforms {
    // time: Uniform<f32>,
    // scale: Uniform<f32>,
    tile_amounts: Uniform<Vector2<i32>>,
    tilemap: Uniform<Texture2D>,
    positions: ShaderStorageBuffer<Vector2<f32>>,
    uvs: ShaderStorageBuffer<Vector2<f32>>,
    fg_colors: ShaderStorageBuffer<Vector4<f32>>,
    bg_colors: ShaderStorageBuffer<Vector4<f32>>,
}

use std::collections::HashMap;

struct WorldRenderer {
    program: Program<Vector2<f32>, WorldUniforms>,
    vao: VertexArray,
    vbo: VertexBuffer<Vector2<f32>>,
    pos_to_index: HashMap<Vector2<usize>, usize>,
    tilemap: Texture2D,
    time: f32,
}

impl WorldRenderer {
    pub fn new(mut program: Program<Vector2<f32>, WorldUniforms>, tilemap: Texture2D) -> Self {
        let mut vao = VertexArray::new();
        let mut vbo = VertexBuffer::new();
        vbo.upload(
            &[
                Vector2::new(0.0, 0.0),
                Vector2::new(0.0, 1.0),
                Vector2::new(1.0, 1.0),
                Vector2::new(0.0, 0.0),
                Vector2::new(1.0, 1.0),
                Vector2::new(1.0, 0.0),
            ],
            UsageType::StaticDraw,
        ).unwrap();
        vao.add_buffer(&vbo).unwrap();
        WorldRenderer {
            program,
            vao,
            vbo,
            tilemap,
            pos_to_index: HashMap::new(),
            time: 0.0,
        }
    }
}

impl<'a> System<'a> for WorldRenderer {
    type SystemData = (
        Read<'a, NewTerrain>,
        Read<'a, ModifiedTerrain>,
        ReadStorage<'a, Terrain>,
        ReadStorage<'a, TerrainColor>,
        ReadStorage<'a, TerrainSprite>,
    );
    fn run(&mut self, (new, modified, pos, color, sprite): Self::SystemData) {
        let &ModifiedTerrain((was_modified, ref modified_set)) = &*modified;
        let &NewTerrain((was_new, ref new_set)) = &*new;
        let env = self.program.env_mut();

        // env.offset.set(Vector2::new(0.0, 0.0));
        // env.scale.set(1.0);
        env.tile_amounts.set(&Vector2::new(MAP_WIDTH as i32, MAP_HEIGHT as i32));

        // IDEA: After passing some threshold, should we just re-upload the buffers?
        // New tiles, reupload all the GPU buffers with the new expanded data set
        if was_new {
            let positions = pos.join().map(|&Terrain(val)| val.cast().unwrap()).collect::<Vec<Vector2<f32>>>();
            let fgs = color.join().map(|&TerrainColor { fg, .. }| fg).collect::<Vec<_>>();
            let bgs = color.join().map(|&TerrainColor { bg, .. }| bg).collect::<Vec<_>>();
            let sprites = sprite.join().map(|&TerrainSprite(val)| val.sprite()).collect::<Vec<_>>();

            self.pos_to_index.clear();

            // Update the pos -> buffer index map; we use this for updating tiles
            for (index, &pos) in positions.iter().enumerate() {
                self.pos_to_index.insert(pos.cast().unwrap(), index);
            }

            env.positions.upload(&*positions, UsageType::DynamicDraw).unwrap();
            env.fg_colors.upload(&*fgs, UsageType::DynamicDraw).unwrap();
            env.bg_colors.upload(&*bgs, UsageType::DynamicDraw).unwrap();
            env.uvs.upload(&*sprites, UsageType::DynamicDraw).unwrap();
        }

        // Modified data, we gotta map the buffer and update everything.
        if was_modified {
            let mut positions = env.positions.map_mut().unwrap().unwrap();
            let mut fg_colors = env.fg_colors.map_mut().unwrap().unwrap();
            let mut bg_colors = env.bg_colors.map_mut().unwrap().unwrap();

            for (&Terrain(pos), &TerrainColor { fg, bg }) in (&pos, &color).join() {
                let idx = self.pos_to_index[&pos];
                positions[idx] = pos.cast().unwrap();
                fg_colors[idx] = fg;
                bg_colors[idx] = bg;
            }
        }

        // TODO: cleaner rendering solution (aka cleaned up draw calls)
        unsafe {
            self.vao.bind();
            self.vbo.bind();
            self.program.bind();
            self.tilemap.bind();
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl_call!(DrawArraysInstanced(
                gl::TRIANGLES,
                0,
                self.vbo.len() as i32,
                (MAP_WIDTH * MAP_HEIGHT) as i32
            )).unwrap();
        }

        self.time += 0.01;
    }
}

struct GridTracker {
    new_id: ReaderId<InsertedFlag>,
    modified_id: ReaderId<ModifiedFlag>,
}

impl<'a> System<'a> for GridTracker {
    type SystemData = (
        Write<'a, NewTerrain>,
        Write<'a, ModifiedTerrain>,
        Write<'a, TileGrid, PanicHandler>,
        Entities<'a>,
        ReadStorage<'a, Terrain>,
        ReadStorage<'a, TerrainColor>,
    );

    fn run(&mut self, (mut new, mut modified, mut grid, entities, pos, colors): Self::SystemData) {
        (new.0).1.clear();
        (modified.0).1.clear();

        // We need to figure out if there were any insertions/modifications,
        // and this seems like this is the only way...
        let inserted_iter = pos.inserted().read(&mut self.new_id);
        (new.0).0 = inserted_iter.len() > 0;
        (new.0).1.extend(inserted_iter.map(|item| *item.as_ref()));

        let modified_iter = colors.modified().read(&mut self.modified_id);
        (modified.0).0 = modified_iter.len() > 0;
        (modified.0).1.extend(modified_iter.map(|item| *item.as_ref()));

        for (entity, &Terrain(pos)) in (&*entities, &pos).join() {
            let (width, height) = grid.dimensions();
            if pos.x < width && pos.y < height {
                // Position is in bounds, update the reference map.
                grid[pos] = entity;
            }
        }
    }
}

struct TileDemoSystem;

impl<'a> System<'a> for TileDemoSystem {
    type SystemData = (Read<'a, TileGrid, PanicHandler>, WriteStorage<'a, TerrainColor>);
    fn run(&mut self, (grid, mut colors): Self::SystemData) {
        // let grid = &**grid;

        // let pos_x = rand::thread_rng().gen_range::<usize>(0, MAP_WIDTH);
        // let pos_y = rand::thread_rng().gen_range::<usize>(0, MAP_HEIGHT);

        // if let Some(color) = colors.get_mut(grid[Vector2::new(pos_x, pos_y)]) {
        //     *color = TerrainColor { fg: Vector4::new(1.0, 1.0, 0.0, 1.0), bg: Vector4::new(0.0, 1.0, 1.0, 1.0) };
        // }
    }
}

fn place_room(world: &mut World, p1: Vector2<usize>, p2: Vector2<usize>) {
    use std::cmp::{min, max};
    use tiles::*;
    let (min_x, min_y) = (min(p1.x, p2.x), min(p1.y, p2.y));
    let (max_x, max_y) = (max(p1.x, p2.x), max(p1.y, p2.y));
    let entity_grid = world.res.fetch::<TileGrid>();
    let mut sprites = world.write_storage::<TerrainSprite>();

    let mut set_border = |x, y, border| {
        let tile = sprites.get_mut(entity_grid[Vector2::new(x, y)]).unwrap();
        println!("{:?}", tile);
        if let Some(prev) = tile.as_border() {
            let new = prev | border;
            if border == BORDER_BEND_BOTTOM_LEFT || prev == BORDER_BEND_BOTTOM_LEFT { print!("BOTTOM LEFT "); }
            if border == BORDER_BEND_BOTTOM_RIGHT || prev == BORDER_BEND_BOTTOM_RIGHT { print!("BOTTOM RIGHT "); }
            if border == BORDER_BEND_TOP_LEFT || prev == BORDER_BEND_TOP_LEFT { print!("TOP LEFT "); }
            if border == BORDER_BEND_TOP_RIGHT || prev == BORDER_BEND_TOP_RIGHT { print!("TOP RIGHT "); }
            println!("OVERLAP: ({}, {}) {:?} -> {:?}", x, y, prev, new);
            *tile = TerrainSprite(Tile::Border(new));
        } else {
            *tile = TerrainSprite(Tile::Border(border));
            println!("PLACE: ({}, {}) {:?}", x, y, border);
        }
        // let border = tile.as_border().unwrap_or(BorderTile::empty()) | border;
    };

    for x in min(min_x + 1, max_x)..max_x {
        // Bottom and top
        set_border(x, min_y, BORDER_VERTICAL);
        set_border(x, max_y, BORDER_VERTICAL);
    }

    for y in min(min_y + 1, max_y)..max_y {
        // Left and right
        set_border(min_x, y, BORDER_HORIZONTAL);
        set_border(max_x, y, BORDER_HORIZONTAL);
    }

    set_border(min_x, min_y, BORDER_BEND_BOTTOM_LEFT);
    set_border(min_x, max_y, BORDER_BEND_TOP_LEFT);
    set_border(max_x, min_y, BORDER_BEND_BOTTOM_RIGHT);
    set_border(max_x, max_y, BORDER_BEND_TOP_RIGHT);
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Birblike")
        .with_dimensions(1000, 1000);
    let context = glutin::ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (4, 3)))
        .with_vsync(true);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe {
        gl_window.make_current().unwrap();
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
        gl::ClearColor(0.5, 0.5, 0.5, 1.0);
        // gl::Enable(gl::BLEND);
        // gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    let vertex = Shader::new(ShaderType::Vertex).unwrap();
    let fragment = Shader::new(ShaderType::Fragment).unwrap();

    vertex.source_from_file("res/world_ssbo.glslv").unwrap();
    fragment.source_from_file("res/world.glslf").unwrap();

    let mut program: Program<Vector2<f32>, _> = ProgramBuilder::new(vertex, fragment)
        .unwrap()
        .build(|mut builder| {
            Ok(WorldUniforms {
                // time: builder.uniform("time")?,
                // scale: builder.uniform("scale")?,
                tilemap: builder.uniform("tilemap")?,
                tile_amounts: builder.uniform("tile_amounts")?,
                uvs: builder.shader_storage("uvs")?,
                positions: builder.shader_storage("positions")?,
                fg_colors: builder.shader_storage("fg_colors")?,
                bg_colors: builder.shader_storage("bg_colors")?,
            })
        })
        .expect("blah");

    use specs::DispatcherBuilder;

    let mut world = World::new();

    world.register::<TilePos>();
    world.register::<TerrainSprite>();
    world.register::<TerrainColor>();
    world.register::<Terrain>();

    let new_id = world.write_storage::<Terrain>().track_inserted();
    let modified_id = world.write_storage::<TerrainColor>().track_modified();

    let mut rng = rand::thread_rng();
    let mut entity_refs = vec![];
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let entity = world
                .create_entity()
                .with(Terrain(Vector2::new(x, y)))
                .with(TerrainSprite(Tile::Air))
                .with(TerrainColor {
                    fg: Vector4::new(1.0, 1.0, 1.0, 1.0),
                    bg: Vector4::new(0.0, 0.0, 0.0, 1.0),
                })
                .build();
            entity_refs.push(entity);
        }
    }

    world.add_resource(ModifiedTerrain((false, BitSet::new())));
    world.add_resource(NewTerrain((false, BitSet::new())));
    world.add_resource(TileGrid(GridX::from_iter(
        entity_refs,
        MAP_WIDTH,
        MAP_HEIGHT,
    )));

    let texture = Texture2D::new();
    texture.set_texture_bank(0);
    let mut image = image::open("res/tileset.bmp").unwrap().flipv().to_rgba();
    for (_, _, pixel) in image.enumerate_pixels_mut() {
        if pixel.data == [255, 0, 255, 255] {
            pixel.data = [0, 0, 0, 0];
        }
    }
    texture.source(image::DynamicImage::ImageRgba8(image)).unwrap();
    texture.mag_filter(MagnificationFilter::Nearest);
    texture.min_filter(MinimizationFilter::Linear);
    texture.texture_wrap_behavior(TextureAxis::S, WrapMode::Repeat);
    texture.texture_wrap_behavior(TextureAxis::T, WrapMode::Repeat);

    program.env_mut().tilemap.set(&texture);

    let mut dispatcher = DispatcherBuilder::new()
        .with(GridTracker { new_id, modified_id }, "track_grid", &[])
        .with(TileDemoSystem, "demo", &["track_grid"])
        .with_thread_local(WorldRenderer::new(program, texture))
        .build();

    // {
    //     let mut rng = rand::thread_rng();

    //     for _ in 0..5 {
    //         let x1 = rng.gen_range(0, MAP_WIDTH);
    //         let x2 = rng.gen_range(0, MAP_WIDTH);
    //         let y1 = rng.gen_range(0, MAP_HEIGHT);
    //         let y2 = rng.gen_range(0, MAP_HEIGHT);
    //         place_room(&mut world, Vector2::new(x1, y1), Vector2::new(x2, y2));
    //     }
    // }

    place_room(&mut world, Vector2::new(2, 2), Vector2::new(12, 12));
    place_room(&mut world, Vector2::new(2, 4), Vector2::new(12, 14));
    // place_room(&mut world, Vector2::new(4, 2), Vector2::new(14, 12));
    // place_room(&mut world, Vector2::new(4, 4), Vector2::new(14, 14));

    let mut running = true;
    while running {
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => running = false,
                glutin::WindowEvent::Resized(w, h) => gl_window.resize(w, h),
                _ => (),
            },
            _ => (),
        });


        dispatcher.dispatch(&mut world.res);
        world.maintain();
        gl_window.swap_buffers().unwrap();
    }
}
