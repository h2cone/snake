use bevy::{
    math::{ivec3, uvec2},
    prelude::*,
};
use bevy_simple_tilemap::{plugin::SimpleTileMapPlugin, Tile, TileFlags, TileMap};

use rand::prelude::*;

const TILE_SCALE: f32 = 2.;
const TILE_WIDTH: f32 = 16. * TILE_SCALE;
const TILE_HEIGHT: f32 = 16. * TILE_SCALE;
const TILE_ROWS: i32 = 12;
const TILE_COLUMNS: i32 = 12;

const SNAKE_TIMER_DURATION: f32 = 0.4;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Snake".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(SimpleTileMapPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, movment)
        .add_systems(Update, turn)
        .run();
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component)]
struct Snake {
    segments: Vec<IVec3>,
    direction: Direction,
}

impl Snake {
    fn head(&self) -> IVec3 {
        *self.segments.last().expect("Snake without head")
    }

    fn grow(&mut self) {
        let head = self.head();
        match self.direction {
            Direction::Up => self.segments.push(ivec3(head.x, head.y + 1, 0)),
            Direction::Down => self.segments.push(ivec3(head.x, head.y - 1, 0)),
            Direction::Left => self.segments.push(ivec3(head.x - 1, head.y, 0)),
            Direction::Right => self.segments.push(ivec3(head.x + 1, head.y, 0)),
        }
    }
}

#[derive(Resource)]
struct SnakeTimer(Timer);

#[derive(Component)]
struct Food {
    position: IVec3,
}

fn movment(
    mut foods: Query<&mut Food>,
    time: Res<Time>,
    mut timer: ResMut<SnakeTimer>,
    mut snakes: Query<&mut Snake>,
    mut tiles: Query<&mut TileMap>,
) {
    if timer.0.tick(time.delta()).finished() {
        let mut food = foods.single_mut();
        let mut tilemap = tiles.single_mut();
        let mut snake = snakes.single_mut();
        let mut new_pos = snake.head();
        match snake.direction {
            Direction::Up => new_pos.y += 1,
            Direction::Down => new_pos.y -= 1,
            Direction::Left => new_pos.x -= 1,
            Direction::Right => new_pos.x += 1,
        }
        let is_over = snake.segments.contains(&new_pos)
            || new_pos.x < 0
            || new_pos.x >= TILE_COLUMNS
            || new_pos.y < 0
            || new_pos.y >= TILE_ROWS;
        if is_over {
            return;
        }
        if new_pos == food.position {
            snake.grow();
            generate_food(&snake.segments).map(|food_pos| {
                food.position = food_pos;
                tilemap.set_tile(
                    food_pos,
                    Some(Tile {
                        sprite_index: 1,
                        ..default()
                    }),
                );
            });
        }
        update_snake(new_pos, &mut snake, &mut tilemap);
    }
}

fn turn(input: Res<ButtonInput<KeyCode>>, mut snakes: Query<&mut Snake>) {
    for mut snake in snakes.iter_mut() {
        match snake.direction {
            Direction::Up | Direction::Down => {
                if input.just_pressed(KeyCode::ArrowLeft) {
                    snake.direction = Direction::Left;
                }
                if input.just_pressed(KeyCode::ArrowRight) {
                    snake.direction = Direction::Right;
                }
            }
            Direction::Left | Direction::Right => {
                if input.just_pressed(KeyCode::ArrowUp) {
                    snake.direction = Direction::Up;
                }
                if input.just_pressed(KeyCode::ArrowDown) {
                    snake.direction = Direction::Down;
                }
            }
        }
    }
}

fn update_snake(new_pos: IVec3, snake: &mut Snake, tilemap: &mut TileMap) {
    for i in 0..snake.segments.len() - 1 {
        let next = snake.segments[i + 1];
        let current = snake.segments[i];
        tilemap.set_tile(
            current,
            Some(Tile {
                sprite_index: 0,
                ..default()
            }),
        );
        snake.segments[i] = next;
    }
    snake.segments.pop();
    for seg in snake.segments.iter() {
        tilemap.set_tile(
            *seg,
            Some(Tile {
                sprite_index: 2,
                ..default()
            }),
        );
    }
    snake.segments.push(new_pos);
    let (idx, flags) = match snake.direction {
        Direction::Up => (3, TileFlags::default()),
        Direction::Down => (3, TileFlags::FLIP_Y),
        Direction::Left => (4, TileFlags::FLIP_X),
        Direction::Right => (4, TileFlags::default()),
    };
    tilemap.set_tile(
        new_pos,
        Some(Tile {
            sprite_index: idx,
            flags,
            ..default()
        }),
    );
}

fn startup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2d::default());

    let image = asset_server.load("textures/tilesheet.png");
    let atlas = TextureAtlasLayout::from_grid(uvec2(16, 16), 5, 1, None, None);
    let atlas_handle = texture_atlases.add(atlas);

    let mut tiles = Vec::new();
    for x in 0..TILE_COLUMNS {
        for y in 0..TILE_ROWS {
            tiles.push((
                ivec3(x, y, 0),
                Some(Tile {
                    sprite_index: 0,
                    ..default()
                }),
            ));
        }
    }
    let mut tilemap = TileMap::new(image, atlas_handle);
    tilemap.set_tiles(tiles);

    // snake head pos
    let head = ivec3(0, 2, 0);
    tilemap.set_tile(
        head,
        Some(Tile {
            sprite_index: 3,
            ..default()
        }),
    );
    let mut segments = vec![ivec3(0, 0, 0), ivec3(0, 1, 0)];
    for seg in segments.iter() {
        tilemap.set_tile(
            *seg,
            Some(Tile {
                sprite_index: 2,
                ..default()
            }),
        );
    }
    segments.push(head);

    // food pos
    let food = generate_food(&segments).expect("No food position");
    tilemap.set_tile(
        food,
        Some(Tile {
            sprite_index: 1,
            ..default()
        }),
    );

    commands
        .spawn((
            tilemap,
            Transform {
                scale: Vec3::splat(TILE_SCALE),
                // centering the tiles is equivalent to moving the tiles’ center to the center of their bottom-left tile
                // m/2 + x = M/2 -> x = M/2 - m/2
                translation: Vec3::new(
                    -(((TILE_WIDTH * TILE_COLUMNS as f32) / 2.) - (TILE_WIDTH / 2.)),
                    -(((TILE_HEIGHT * TILE_ROWS as f32) / 2.) - (TILE_HEIGHT / 2.)),
                    0.,
                ),
                ..default()
            },
        ))
        .insert(Snake {
            segments,
            direction: Direction::Up,
        })
        .insert(Food { position: food });

    commands.insert_resource(SnakeTimer(Timer::from_seconds(
        SNAKE_TIMER_DURATION,
        TimerMode::Repeating,
    )));
}

fn generate_food(segments: &[IVec3]) -> Option<IVec3> {
    if segments.len() == TILE_ROWS as usize * TILE_COLUMNS as usize {
        return None;
    }
    let mut rng = rand::rng();
    let x = rng.random_range(0..TILE_COLUMNS);
    let y = rng.random_range(0..TILE_ROWS);
    let food = ivec3(x, y, 0);
    if segments.contains(&food) {
        return generate_food(segments);
    }
    Some(food)
}
