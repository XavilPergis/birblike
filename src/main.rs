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
    ($name:ident is $type:ty) => {
        pub struct $name($type);
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
}

use grid::GridX;

#[derive(Debug)]
pub struct TileGrid(GridX<Entity>);
#[derive(Debug)]
pub struct PreviousTileGrid(GridX<Entity>);
#[derive(Debug)]
pub struct ChangeSet(HashMap<Vector2<usize>, Entity>);

pub struct TilePos(Vector2<usize>);
pub struct TileColor(Vector4<f32>);

#[derive(Default)]
pub struct Terrain;

use specs::prelude::*;

impl Component for TilePos {
    type Storage = VecStorage<TilePos>;
}

impl Component for TileColor {
    type Storage = VecStorage<TileColor>;
}

impl Component for Terrain {
    type Storage = NullStorage<Terrain>;
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
    type SystemData = (ReadStorage<'a, TilePos>, ReadStorage<'a, TileColor>);
    fn run(&mut self, (pos, color): Self::SystemData) {
        let env = self.program.environment_mut();

        // env.offset.set(Vector2::new(0.0, 0.0));
        // env.scale.set(1.0);
        env.tile_amounts.set(Vector2::new(100, 100));

        let positions = pos.join().map(|&TilePos(val)| val.cast().unwrap()).collect::<Vec<_>>();
        let colors = color.join().map(|&TileColor(val)| val).collect::<Vec<_>>();

        env.positions.upload(&*positions, UsageType::DynamicDraw).unwrap();
        env.colors.upload(&*colors, UsageType::DynamicDraw).unwrap();

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

#[derive(Default)]
struct GridTracker {
    previous_grid: Option<GridX<Entity>>,
}

impl<'a> System<'a> for GridTracker {
    type SystemData = (
        Write<'a, TileGrid, PanicHandler>,
        // Write<'a, ChangeSet, PanicHandler>,
        Entities<'a>,
        ReadStorage<'a, TilePos>,
        ReadStorage<'a, Terrain>,
    );
    fn run(&mut self, (mut grid, entities, pos, _): Self::SystemData) {
        let &mut TileGrid(ref mut grid) = &mut *grid;

        if let Some(ref mut prev_grid) = self.previous_grid {
            prev_grid.copy_grid(&grid);
        } else {
            self.previous_grid = Some(grid.clone());
        }

        for (entity, &TilePos(pos)) in (&*entities, &pos).join() {
            let (width, height) = grid.dimensions();
            if pos.x < width && pos.y < height {
                // Position is in bounds, update the reference map.
                grid[pos] = entity;
            }
        }

        // Previous grid should always be initalized, so we can unwrap
        // grid.iter()
        //     .zip(self.previous_grid.as_ref().unwrap().iter())
        //     .filter(|(a, b)| a != b);
    }
}

struct TileDemoSystem;

impl<'a> System<'a> for TileDemoSystem {
    type SystemData = (Read<'a, TileGrid, PanicHandler>, WriteStorage<'a, TileColor>);
    fn run(&mut self, (grid, mut colors): Self::SystemData) {
        let &TileGrid(ref grid) = &*grid;

        let pos_x = rand::thread_rng().gen_range::<usize>(0, 100);
        let pos_y = rand::thread_rng().gen_range::<usize>(0, 100);

        if let Some(color) = colors.get_mut(grid[Vector2::new(pos_x, pos_y)]) {
            *color = TileColor(Vector4::unit_y());
        }
    }
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Birblike")
        .with_dimensions(1024, 768);
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
    world.register::<TileColor>();
    world.register::<Terrain>();

    let (width, height) = (100, 100);

    let mut entity_refs = vec![];
    for y in 0..height {
        for x in 0..width {
            let entity = world
                .create_entity()
                .with(Terrain)
                .with(TilePos(Vector2::new(x, y)))
                .with(TileColor(Vector4::new(
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
    world.add_resource(TileGrid(GridX::from_iter(
        entity_refs,
        width as usize,
        height as usize,
    )));

    let mut dispatcher = DispatcherBuilder::new()
        .with(GridTracker::default(), "track_grid", &[])
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

        // program.environment_mut().time.set(time);

        // time += 0.01;
        //
        world.maintain();
        dispatcher.dispatch(&mut world.res);
        gl_window.swap_buffers().unwrap();
    }
}
