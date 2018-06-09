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

newtype!(TileGrid is GridX<Entity>);
newtype!(PreviousTileGrid is GridX<Entity>);
newtype!(TilePos is Vector2<usize>);
newtype!(Terrain is Vector2<usize>);
newtype!(TerrainColor is Vector4<f32>);
newtype!(#[derive(Default)] ModifiedTerrain is (bool, BitSet));
newtype!(#[derive(Default)] NewTerrain is (bool, BitSet));

#[derive(Debug)]
pub struct ChangeSet(HashMap<Vector2<usize>, Entity>);

#[derive(Default)]
pub struct Dirty;

use specs::prelude::*;

impl Component for TilePos {
    type Storage = VecStorage<Self>;
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
    positions: ShaderStorageBuffer<Vector2<f32>>,
    colors: ShaderStorageBuffer<Vector4<f32>>,
}

use std::collections::HashMap;

struct WorldRenderer {
    program: Program<Vector2<f32>, WorldUniforms>,
    vao: VertexArray,
    vbo: VertexBuffer<Vector2<f32>>,
    pos_to_index: HashMap<Vector2<usize>, usize>,
    time: f32,
}

impl WorldRenderer {
    pub fn new(mut program: Program<Vector2<f32>, WorldUniforms>) -> Self {
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
        ReadStorage<'a, TerrainColor>
    );
    fn run(&mut self, (new, modified, pos, color): Self::SystemData) {
        let &ModifiedTerrain((was_modified, ref modified_set)) = &*modified;
        let &NewTerrain((was_new, ref new_set)) = &*new;
        let env = self.program.env_mut();

        // env.offset.set(Vector2::new(0.0, 0.0));
        // env.scale.set(1.0);
        env.tile_amounts.set(Vector2::new(100, 100));

        // IDEA: After passing some threshold, should we just re-upload the buffers?
        // New tiles, reupload all the GPU buffers with the new expanded data set
        if was_new {
            let positions = pos.join().map(|&Terrain(val)| val.cast().unwrap()).collect::<Vec<Vector2<f32>>>();
            let colors = color.join().map(|&TerrainColor(val)| val).collect::<Vec<_>>();

            self.pos_to_index.clear();

            // Update the pos -> buffer index map; we use this for updating tiles
            for (index, &pos) in positions.iter().enumerate() {
                self.pos_to_index.insert(pos.cast().unwrap(), index);
            }

            env.positions.upload(&*positions, UsageType::DynamicDraw).unwrap();
            env.colors.upload(&*colors, UsageType::DynamicDraw).unwrap();
        }

        // Modified data, we gotta map the buffer and update everything.
        if was_modified {
        {
            let mut positions = env.positions.map_mut().unwrap().unwrap();
            let mut colors = env.colors.map_mut().unwrap().unwrap();

            for (&Terrain(pos), &TerrainColor(color)) in (&pos, &color).join() {
                let idx = self.pos_to_index[&pos];
                positions[idx] = pos.cast().unwrap();
                colors[idx] = color;
            }
        }
        }

        // TODO: cleaner rendering solution
        unsafe {
            self.vao.bind();
            self.vbo.bind();
            self.program.bind();
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl_call!(DrawArraysInstanced(
                gl::TRIANGLES,
                0,
                self.vbo.len() as i32,
                100 * 100
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

        pos.populate_modified(&mut self.modified_id, &mut (modified.0).1);
        // (modified.0).0 = true;

        // let modified_iter = pos.modified().read(&mut self.modified_id);
        // (modified.0).0 = modified_iter.len() > 0;
        // (modified.0).1.extend(modified_iter.map(|item| *item.as_ref()));

        for (entity, &Terrain(pos), &TerrainColor(color)) in (&*entities, &pos, &colors).join() {
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
        let grid = &**grid;

        let pos_x = rand::thread_rng().gen_range::<usize>(0, 100);
        let pos_y = rand::thread_rng().gen_range::<usize>(0, 100);

        if let Some(color) = colors.get_mut(grid[Vector2::new(pos_x, pos_y)]) {
            *color = TerrainColor(Vector4::unit_y());
        }
    }
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
                tile_amounts: builder.uniform("tile_amounts")?,
                positions: builder.shader_storage("positions")?,
                colors: builder.shader_storage("colors")?,
            })
        })
        .expect("blah");

    use specs::DispatcherBuilder;

    let mut world = World::new();

    world.register::<TilePos>();
    world.register::<TerrainColor>();
    world.register::<Terrain>();

    let new_id = world.write_storage::<Terrain>().track_inserted();
    let modified_id = { let mut ws = world.write_storage::<TerrainColor>(); ws.track_modified() };

    let (width, height) = (100, 100);

    let mut entity_refs = vec![];
    for y in 0..height {
        for x in 0..width {
            let entity = world
                .create_entity()
                .with(Terrain(Vector2::new(x, y)))
                .with(TerrainColor(Vector4::new(
                    x as f32 / 100.0,
                    0.0,
                    y as f32 / 100.0,
                    1.0,
                )))
                .build();
            entity_refs.push(entity);
        }
    }

    world.add_resource(PreviousTileGrid(GridX::from_iter(std::iter::empty(), 0, 0)));
    world.add_resource(ModifiedTerrain((false, BitSet::new())));
    world.add_resource(NewTerrain((false, BitSet::new())));
    world.add_resource(TileGrid(GridX::from_iter(
        entity_refs,
        width as usize,
        height as usize,
    )));

    let mut dispatcher = DispatcherBuilder::new()
        .with(GridTracker { new_id, modified_id }, "track_grid", &[])
        .with(TileDemoSystem, "demo", &["track_grid"])
        .with_thread_local(WorldRenderer::new(program))
        .build();

    let mut time = 0.0;
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

        world.maintain();
        dispatcher.dispatch(&mut world.res);
        gl_window.swap_buffers().unwrap();
    }
}
