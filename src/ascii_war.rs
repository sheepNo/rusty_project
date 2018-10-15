use std::io;

const MAX_X: u32 = 7;
const MAX_Y: u32 = 7;
const NB_CHAR: usize = 2;

#[derive(Debug)]
enum Direction {
    North,
    South,
    East,
    West,
    Stay,
}

#[derive(Debug)]
enum Status {
    Alive,
    Dead,
}

#[derive(Debug, Copy, Clone)]
enum CellState {
    Empty,
    Wall,
    Char,
}

#[derive(Debug, Copy, Clone)]
struct Cell {
    state: CellState,
    id: usize,
}
impl Cell {
    fn is_empty(&self) -> bool {
        match self.state {
            CellState::Empty => true,
            _ => false,
        }
    }
    fn disp(&self) -> String {
        match self.state {
            CellState::Empty => String::from(" 0 "),
            CellState::Wall => String::from(" X "),
            CellState::Char => String::from(" @ "),
        }
    }
}

fn parse(mov: &str) -> Direction {
    match mov.chars().next() {
        Some('d') => Direction::North,
        Some('s') => Direction::South,
        Some('r') => Direction::West,
        Some('t') => Direction::East,
        _ => Direction::Stay,
    }
}

fn display_map(map: &mut [Cell]) {
    for y in 0..=MAX_Y {
        for x in 0..=MAX_X {
            print!("{}", map[(x + y * (MAX_X+1)) as usize].disp());
        }
        println!("");
    }
}

#[derive(Debug)]
struct Character {
    id: usize,
    x: u32,
    y: u32,
    mobi: u8,
    facing: Direction,
    hp: u8,
    state: Status,
}
impl Character {
    fn mv(&mut self, map: &mut [Cell], mov: &str) {
        println!("old pos = ({},{})", self.x, self.y);
        match parse(mov) {
            Direction::South if self.y < MAX_Y && map[(self.x + (self.y+1) * (MAX_X+1)) as usize].is_empty() => {
                map[(self.x + self.y * (MAX_X+1)) as usize].state = CellState::Empty;
                map[(self.x + (self.y+1) * (MAX_X+1)) as usize].state = CellState::Char;
                map[(self.x + (self.y+1) * (MAX_X+1)) as usize].id = self.id;
                self.y = self.y + 1;
                println!("Moving south")
            },
            Direction::North if self.y > 0 && map[(self.x + (self.y-1) * (MAX_X+1)) as usize].is_empty() => {
                map[(self.x + self.y * (MAX_X+1)) as usize].state = CellState::Empty;
                map[(self.x + (self.y-1) * (MAX_X+1)) as usize].state = CellState::Char;
                map[(self.x + (self.y-1) * (MAX_X+1)) as usize].id = self.id;
                self.y = self.y - 1;
                println!("Moving north")
            },
            Direction::West if self.x < MAX_X && map[((self.x+1) + self.y * (MAX_X+1)) as usize].is_empty() => {
                map[(self.x + self.y * (MAX_X+1)) as usize].state = CellState::Empty;
                map[((self.x+1) + self.y * (MAX_X+1)) as usize].state = CellState::Char;
                map[((self.x+1) + self.y * (MAX_X+1)) as usize].id = self.id;
                self.x = self.x + 1;
                println!("Moving west")
            },
            Direction::East if self.x > 0 && map[((self.x-1) + self.y * (MAX_X+1)) as usize].is_empty() => {
                map[(self.x + self.y * (MAX_X+1)) as usize].state = CellState::Empty;
                map[((self.x-1) + self.y * (MAX_X+1)) as usize].state = CellState::Char;
                map[((self.x-1) + self.y * (MAX_X+1)) as usize].id = self.id;
                self.x = self.x - 1;
                println!("Moving east")
            },
            _ => println!("Can't move this way."),
        };
        println!("new pos = ({},{})", self.x, self.y);
    }
}


fn main() {
    let mut char_id = 0;

    let mut char_list = [
        Character {
            id: 0,
            x: 3,
            y: 0,
            mobi: 3,
            hp: 3,
            facing: Direction::North,
            state: Status::Alive,
        },
        Character {
            id: 1,
            x: 6,
            y: 7,
            mobi: 2,
            hp: 5,
            facing: Direction::South,
            state: Status::Alive,
        }
    ];

    let mut map = [Cell{state: CellState::Empty, id: 0}; ((MAX_X+1)*(MAX_Y+1)) as usize];
    map[3+2*8].state = CellState::Wall;
    map[3+3*8].state = CellState::Wall;
    map[3+4*8].state = CellState::Wall;
    map[3+0*8].state = CellState::Char;
    map[3+0*8].id = 0;
    map[6+7*8].state = CellState::Char;
    map[6+7*8].id = 1;

    println!("Start!!");

    let mut turn_id = 0;

    loop {
        turn_id += 1;
        println!("|||=======================|||");
        println!("           Turn {}", turn_id);
        println!("|||=======================|||");

        for character in char_list.iter_mut() {
            println!("***=======================***");
            println!("       {}'s turn!", char_id);
            println!("***=======================***");

            println!("You can move {} cells!", character.mobi);
            for i in 0..character.mobi {
                display_map(&mut map);

                println!("cell n°{}:", i);
                let mut mov = String::new();
                io::stdin().read_line(&mut mov)
                    .expect("Failed to read line");

                character.mv(&mut map, &mov);
            };
            char_id = (char_id + 1) % NB_CHAR;
        };

        println!("End of turn {}", turn_id);
    };
}
