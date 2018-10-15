extern crate ggez;

// use std::io;
use ggez::event::Keycode;
use ggez::{event, graphics, Context, GameResult};
use ggez::timer;
use std::time::{Duration, Instant};

const GRID_X: u16 = 16;
const GRID_Y: u16 = 16;

const RES_X: u16 = 32;
const RES_Y: u16 = 32;

const UPDATES_PER_SECOND: f32 = 2.0;
const MILLIS_PER_UPDATE: u64 = (1.0 / UPDATES_PER_SECOND * 1000.0) as u64;

const NB_CHAR: usize = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct GridPosition {
    x: u16,
    y: u16,
}

impl GridPosition {
    fn as_index(&self) -> usize {
        (self.x + self.y * GRID_X) as usize
    }
}

impl From<GridPosition> for graphics::Rect {
    fn from(pos: GridPosition) -> Self {
        graphics::Rect::new_i32(
            pos.x as i32 * RES_X as i32,
            pos.y as i32 * RES_Y as i32,
            RES_X as i32,
            RES_Y as i32,
        )
    }
}

/// And here we implement `From` again to allow us to easily convert between
/// `(i16, i16)` and a `GridPosition`.
impl From<(u16, u16)> for GridPosition {
    fn from(pos: (u16, u16)) -> Self {
        GridPosition { x: pos.0, y: pos.1 }
    }
}

#[derive(Debug, Copy, Clone)]
enum TileState {
    Empty,
    Wall,
    Trap,
}

#[derive(Debug, Copy, Clone)]
struct Tile {
    state: TileState,
    cooldown: u32,
    char_id: usize,
}

impl Tile {
    fn is_empty(&self) -> bool {
        match self.state {
            // doesn't prevent the player from going into a trap
            TileState::Wall => false,
            _ => self.char_id == NB_CHAR + 1,
        }
    }

    fn draw(&self, ctx: &mut Context, pos: GridPosition) -> GameResult<()> {
        let color = match self.state {
            TileState::Empty => [0.5, 0.3, 0.3, 1.0],
            TileState::Wall => [0.5, 0.0, 0.0, 1.0],
            TileState::Trap => [0.0, 0.6, 0.2, 1.0],
        };
        graphics::set_color(ctx, color.into())?;
        graphics::rectangle(ctx, graphics::DrawMode::Fill, pos.into())
    }
}

struct Map {
    tiles: [Tile; 16*16],
}

impl Map {
    pub fn new(arr: Tile) -> Self {
        Map {
            tiles: [arr; 16*16],
        }
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        for x in 0..GRID_X {
            for y in 0..GRID_Y {
                self.tiles[(y * GRID_X + x) as usize].draw(ctx, GridPosition{x,y})?
            }
        }
        Ok(())
    }

    fn neighbour(&self, pos: GridPosition, dir: &Direction) -> Tile {
        match dir {
            Direction::Up => self.tiles[pos.as_index() - GRID_X as usize],
            Direction::Down => self.tiles[pos.as_index() + GRID_X as usize],
            Direction::Left => self.tiles[pos.as_index() - 1],
            Direction::Right => self.tiles[pos.as_index() + 1]
        }
    }

    fn is_available(&self, pos: GridPosition, dir: &Direction) -> bool {
        self.neighbour(pos, dir).is_empty()
    }
}

#[derive(Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn from_keycode(key: Keycode) -> Option<Direction> {
        match key {
            Keycode::Up => Some(Direction::Up),
            Keycode::Down => Some(Direction::Down),
            Keycode::Left => Some(Direction::Left),
            Keycode::Right => Some(Direction::Right),
            _ => None,
        }
    }
}

#[derive(Debug)]
enum Status {
    Alive,
    Dead,
}

#[derive(Debug)]
struct Character {
    id: usize,
    pos: GridPosition,
    mobi: u8,
    mp: u8,
    facing: Direction,
    hp: u8,
    state: Status,
    selector: GridPosition,
}

impl Character {
    fn draw_selector(&self, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, [0.2, 0.3, 0.8, 0.4].into())?;
        graphics::rectangle(ctx, graphics::DrawMode::Fill, self.selector.into())
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        // Character color. Let's go with a cool blue (RGBA)
        graphics::set_color(ctx, [0.1, 0.3, 0.8, 1.0].into())?;
        // Then we draw a triangle – which orientation depends on the character's – with the Fill draw mode.
        let points = match self.facing {
            Direction::Up => [
                graphics::Point2::new((self.pos.x * RES_X) as f32 + 0.0, (self.pos.y * RES_Y) as f32 + 32.0),
                graphics::Point2::new((self.pos.x * RES_X) as f32 + 32.0, (self.pos.y * RES_Y) as f32 + 32.0),
                graphics::Point2::new((self.pos.x * RES_X) as f32 + 16.0, (self.pos.y * RES_Y) as f32 + 0.0)
            ],
            Direction::Down => [
                graphics::Point2::new((self.pos.x * RES_X) as f32 + 0.0, (self.pos.y * RES_Y) as f32 + 0.0),
                graphics::Point2::new((self.pos.x * RES_X) as f32 + 32.0, (self.pos.y * RES_Y) as f32 + 0.0),
                graphics::Point2::new((self.pos.x * RES_X) as f32 + 16.0, (self.pos.y * RES_Y) as f32 + 32.0)
            ],
            Direction::Left => [
                graphics::Point2::new((self.pos.x * RES_X) as f32 + 32.0, (self.pos.y * RES_Y) as f32 + 0.0),
                graphics::Point2::new((self.pos.x * RES_X) as f32 + 32.0, (self.pos.y * RES_Y) as f32 + 32.0),
                graphics::Point2::new((self.pos.x * RES_X) as f32 + 0.0, (self.pos.y * RES_Y) as f32 + 16.0)
            ],
            Direction::Right => [
                graphics::Point2::new((self.pos.x * RES_X) as f32 + 0.0, (self.pos.y * RES_Y) as f32 + 0.0),
                graphics::Point2::new((self.pos.x * RES_X) as f32 + 0.0, (self.pos.y * RES_Y) as f32 + 32.0),
                graphics::Point2::new((self.pos.x * RES_X) as f32 + 32.0, (self.pos.y * RES_Y) as f32 + 16.0)
            ],
        };
        graphics::polygon(ctx, graphics::DrawMode::Fill, &points)
    }
}

enum Action {
    Attack,
    Move,
}
/// Now we have the heart of our game, the GameState. This struct
/// will implement ggez's `EventHandler` trait and will therefore drive
/// everything else that happens in our game.
struct GameState {
    /// First we need a character
    characters: [Character;2],
    action: Action,
    ap: u32,
    /// Then a map
    map: Map,
    /// Whether the game is over or not
    gameover: bool,
    turn: u32,
    char_id: usize,
    /// And we track the last time we updated so that we can limit
    /// our update rate.
    last_update: Instant,
}

impl GameState {
    /// Our new function will set up the initial state of our game.
    pub fn new() -> Self {
        // First we put our snake a quarter of the way across our grid in the x axis
        // and half way down the y axis. This works well since we start out moving to the right.
        // Then we choose a random place to put our piece of food using the helper we made
        // earlier.
        // let food_pos = GridPosition::random(GRID_SIZE.0, GRID_SIZE.1);
        let bitmap = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                      1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                      1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                      1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1,
                      1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 2, 2, 1,
                      1, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 1, 2, 2, 0, 1,
                      1, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0, 1,
                      1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1,
                      1, 0, 1, 1, 0, 0, 0, 1, 1, 1, 2, 2, 1, 0, 0, 1,
                      1, 0, 0, 1, 0, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1,
                      1, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                      1, 0, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1,
                      1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1,
                      1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1,
                      1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                      1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        let mut map = Map::new(Tile{state: TileState::Empty, cooldown: 0, char_id: NB_CHAR + 1});
        // map.tiles[10].state = TileState::Wall;
        // map.tiles[11].state = TileState::Wall;
        // map.tiles[19].state = TileState::Wall;
        // map.tiles[20].state = TileState::Wall;
        // map.tiles[22].state = TileState::Wall;
        // map.tiles[34].state = TileState::Wall;
        // map.tiles[42].state = TileState::Wall;
        // map.tiles[50].state = TileState::Wall;
        // for index in 0..16 {
        //     map.tiles[index].state = TileState::Wall;
        //     map.tiles[(GRID_X * GRID_Y - 1) as usize - index].state = TileState::Wall;
        //     map.tiles[index * GRID_X as usize].state = TileState::Wall;
        //     map.tiles[index * GRID_X as usize + GRID_X as usize - 1].state = TileState::Wall;
        // }
        for (index,bit) in bitmap.iter().enumerate() {
            match bit {
                2 => map.tiles[index].state = TileState::Trap,
                1 => map.tiles[index].state = TileState::Wall,
                _ => (),
            }
        }

        GameState {
            characters: [Character {
                             id: 0,
                             pos: GridPosition{x:1,y:1},
                             mobi: 3,
                             mp: 3,
                             facing: Direction::Down,
                             selector: GridPosition{x:1,y:1},
                             hp: 5,
                             state: Status::Alive},
                         Character {
                             id: 1,
                             pos: GridPosition{x:13,y:13},
                             mobi: 3,
                             mp: 3,
                             facing: Direction::Up,
                             selector: GridPosition{x:13,y:13},
                             hp: 5,
                             state: Status::Alive}],
            action: Action::Move,
            ap: 3,
            map: map,
            gameover: false,
            char_id: 0,
            turn: 0,
            last_update: Instant::now(),
        }
    }
}

/// Now we implement EventHandler for GameState. This provides an interface
/// that ggez will call automatically when different events happen.
impl event::EventHandler for GameState {
    /// Update will happen on every frame before it is drawn. This is where we update
    /// our game state to react to whatever is happening in the game world.
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        // const DESIRED_FPS: u32 = 60;
        // // This tries to throttle updates to desired value.
        // while timer::check_update_time(ctx, DESIRED_FPS) {
        //     // Since we don't have any non-callback logic, all we do is append our logs.
        //     self.file_logger.update()?;
        // }
        // Ok(())
        // First we check to see if enough time has elapsed since our last update based on
        // the update rate we defined at the top.
        if Instant::now() - self.last_update >= Duration::from_millis(MILLIS_PER_UPDATE) {
            // Then we check to see if the game is over. If not, we'll update. If so, we'll just do nothing.
            // if !self.gameover {
            //     // Here we do the actual updating of our game world. First we tell the snake to update itself,
            //     // passing in a reference to our piece of food.
            //     // self.snake.update(&self.food);
            //     // Next we check if the snake ate anything as it updated.
            //     // if let Some(ate) = self.snake.ate {
            //         // If it did, we want to know what it ate.
            //         // match ate {
            //             // If it ate a piece of food, we randomly select a new position for our piece of food
            //             // and move it to this new position.
            //             // Ate::Food => {
            //                 // let new_food_pos = GridPosition::random(GRID_SIZE.0, GRID_SIZE.1);
            //                 // self.food.pos = new_food_pos;
            //             // }
            //             // If it ate itself, we set our gameover state to true.
            //             // Ate::Itself => {
            //             //     self.gameover = true;
            //             // }
            //         // }
            //     }
            // }
            // If we updated, we set our last_update to be now
            self.last_update = Instant::now();
        }
        // Finally we return `Ok` to indicate we didn't run into any errors
        Ok(())
    }

    /// draw is where we should actually render the game's current state.
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // First we clear the screen
        graphics::clear(ctx);
        // Draws the tiles
        self.map.draw(ctx)?;
        // Then we tell the characters to draw themselves
        for character in self.characters.iter() {
            character.draw(ctx)?;
        }
        match self.action {
            Action::Attack => self.characters[self.char_id].draw_selector(ctx)?,
            _ => ()
        }
        // Finally we call graphics::present to cycle the gpu's framebuffer and display
        // the new frame we just drew.
        graphics::present(ctx);
        // We yield the current thread until the next update
        ggez::timer::yield_now();
        // And return success.
        Ok(())
    }

    /// key_down_event gets fired when a key gets pressed.
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: Keycode,
        _keymod: event::Mod,
        _repeat: bool,
    ) {
        let character = &mut self.characters[self.char_id];
        println!("Character pos: {:?}", character.pos);
        if character.mp == 0 {
            self.action = Action::Attack;
            character.selector = character.pos;
            character.mp = character.mobi
        }
        match self.action {
            Action::Move => {
                if let Some(dir) = Direction::from_keycode(keycode) {
                    if self.map.is_available(character.pos, &dir) {
                        self.map.tiles[character.pos.as_index()].char_id = NB_CHAR + 1;
                        match dir {
                            Direction::Up => character.pos.y -= 1,
                            Direction::Down => character.pos.y += 1,
                            Direction::Right => character.pos.x += 1,
                            Direction::Left => character.pos.x -= 1,
                        }
                        self.map.tiles[character.pos.as_index()].char_id = self.char_id;
                        character.mp -= 1;
                    }
                    character.facing = dir;
                } else {
                    match keycode {
                        Keycode::Space => {
                            self.action = Action::Attack;
                            character.selector = character.pos;
                            character.mp = character.mobi
                        },
                        _ => (),
                    }
                }
            },
            Action::Attack => {
                if let Some(dir) = Direction::from_keycode(keycode) {
                    match dir {
                        Direction::Up => {
                            character.selector.y -= 1;
                        },
                        Direction::Down => character.selector.y += 1,
                        Direction::Right => character.selector.x += 1,
                        Direction::Left => character.selector.x -= 1,
                    }
                } else {
                    match keycode {
                        Keycode::Space => {
                            // end turn and change the state of the game
                            self.turn += 1;
                            self.action = Action::Move;
                            self.char_id = (self.char_id + 1) % NB_CHAR;
                        },
                        _ => (),
                    }
                }
            },
        }
    }
}


fn main() {
    let ctx = &mut ggez::ContextBuilder::new("ascii_war", "sheep")
        .window_setup(ggez::conf::WindowSetup::default().title("Fight!"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(GRID_X as u32 * RES_X as u32, GRID_Y as u32 * RES_Y as u32))
        .build().expect("Failed to build ggez context");
    graphics::set_background_color(ctx, [0.0, 0.0, 0.0, 0.0].into());

    let state = &mut GameState::new();
    // And finally we actually run our game, passing in our context and state.
    match event::run(ctx, state) {
        // If we encounter an error, we print it before exiting
        Err(e) => println!("Error encountered running game: {}", e),
        // And if not, we print a message saying we ran cleanly. Hooray!
        Ok(_) => println!("Game exited cleanly!"),
    }
}
