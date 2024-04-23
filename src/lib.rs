use minifb::{Key, KeyRepeat, Window};
use rand::prelude::SliceRandom;
use rand::Rng;
use window_rs::WindowBuffer;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MazeConfig {
    /// The color used for the path
    pub path_color: u32,

    /// The color used for the walls
    pub wall_color: u32,
}

impl Default for MazeConfig {
    fn default() -> Self {
        Self {
            path_color: 0,
            wall_color: u32::MAX,
        }
    }
}

impl MazeConfig {
    pub fn generate(&self, buffer: &mut WindowBuffer, rng: &mut impl Rng) {
        buffer.fill(self.wall_color);

        let x = rng.gen_range(0..buffer.width());
        let y = rng.gen_range(0..buffer.height());

        let mut stack = Vec::new();
        stack.push((x, y));
        buffer[(x, y)] = self.path_color;

        while let Some(cell) = stack.pop() {
            let (x, y) = (cell.0 as isize, cell.1 as isize);

            let mut to_explore = Vec::new();

            if let Some(cell) = buffer.get(x - 2, y) {
                if cell == self.wall_color {
                    let (x, y) = (x as usize, y as usize);
                    to_explore.push((x as usize - 2, y as usize));
                }
            }
            if let Some(cell) = buffer.get(x + 2, y) {
                if cell == self.wall_color {
                    let (x, y) = (x as usize, y as usize);
                    to_explore.push((x as usize + 2, y as usize));
                }
            }
            if let Some(cell) = buffer.get(x, y - 2) {
                if cell == self.wall_color {
                    let (x, y) = (x as usize, y as usize);
                    to_explore.push((x as usize, y as usize - 2));
                }
            }
            if let Some(cell) = buffer.get(x, y + 2) {
                if cell == self.wall_color {
                    let (x, y) = (x as usize, y as usize);
                    to_explore.push((x as usize, y as usize + 2));
                }
            }

            if let Some(cell2) = to_explore.choose(rng) {
                if to_explore.len() > 1 {
                    stack.push(cell);
                }
                stack.push(*cell2);
                buffer[*cell2] = self.path_color;
                let wall = middle_point(cell, *cell2);
                buffer[wall] = self.path_color;
            }
        }
    }
}

pub fn start_end_generator(
    buffer: &mut WindowBuffer,
    rng: &mut impl Rng,
    player: &mut Player,
) -> (usize, usize) {
    let mut start_point_ready = false;
    let mut end_point_ready = false;
    loop {
        let start_height = rng.gen_range(0..buffer.height());
        let end_height = rng.gen_range(0..buffer.height());
        let width_max = buffer.width() - 1;

        if start_point_ready == false && buffer[(1, start_height)] == 0 {
            buffer[(0, start_height)] = player.player_color;
            start_point_ready = true;
        }
        if end_point_ready == false && buffer[(&width_max - 1, end_height)] == 0 {
            buffer[(width_max, end_height)] = player.finish_color;
            end_point_ready = true;
        }
        if start_point_ready == true && end_point_ready == true {
            player.end_point = (width_max, end_height);
            player.position = (0, start_height);
            player.previous_spot = (0, start_height);
            return (0, start_height);
        }
    }
}

fn middle_point(a: (usize, usize), b: (usize, usize)) -> (usize, usize) {
    if a.0 != b.0 {
        let min = a.0.min(b.0);
        let max = a.0.max(b.0);

        (min + (max - min) / 2, a.1)
    } else if a.1 != b.1 {
        let min = a.1.min(b.1);
        let max = a.1.max(b.1);

        (a.0, min + (max - min) / 2)
    } else {
        a
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Direction {
    North,
    South,
    East,
    West,
    Still,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Player {
    player_color: u32,
    position: (usize, usize),

    finish_color: u32,
    end_point: (usize, usize),

    direction: Direction,
    previous_spot: (usize, usize),
    maze_config: MazeConfig,

    pub game_over: bool,
}

impl Player {
    pub fn new(
        position: (usize, usize),
        end_point: (usize, usize),
        direction: Direction,
        previous_spot: (usize, usize),
        maze_config: MazeConfig,
        game_over: bool,
    ) -> Self {
        Self {
            player_color: 0x00FF00,
            finish_color: 0xFF00FF,
            position,
            end_point,
            direction,
            previous_spot,
            maze_config,
            game_over,
        }
    }

    pub fn handle_user_input(
        &mut self,
        window: &Window,
        start_point: &(usize, usize),
    ) -> std::io::Result<()> {
        if window.is_key_pressed(Key::Q, KeyRepeat::No) {
            self.reset(*start_point);
        }

        if window.is_key_pressed(Key::Up, KeyRepeat::Yes) {
            self.direction = Direction::North;
        }

        if window.is_key_pressed(Key::Down, KeyRepeat::Yes) {
            self.direction = Direction::South;
        }

        if window.is_key_pressed(Key::Right, KeyRepeat::Yes) {
            self.direction = Direction::East;
        }

        if window.is_key_pressed(Key::Left, KeyRepeat::Yes) {
            self.direction = Direction::West;
        }

        /*let small_break = Duration::from_millis(0);
        if self.small_break_timer.elapsed() >= small_break {
            window.get_keys_released().iter().for_each(|key| match key {
                Key::Space => self.space_count += 1,
                _ => (),
            });
            self.small_break_timer = Instant::now();
        }*/

        Ok(())
    }

    pub fn reset(&mut self, start_point: (usize, usize)) {
        self.position = start_point;
        self.direction = Direction::Still;
    }

    pub fn direction(&mut self, buffer: &WindowBuffer) {
        let x = self.position.0;
        let y = self.position.1;
        match self.direction {
            Direction::East => {
                if buffer.get(x as isize + 1, y as isize) != None
                    && buffer[(x + 1, y)] != self.maze_config.wall_color
                {
                    if (x + 1, y) == self.end_point {
                        println!("Congrats, you've finished the maze!");
                        self.position = (x + 1, y);
                        self.direction = Direction::Still;
                        self.previous_spot = (x, y);
                        self.game_over = true;
                    } else {
                        self.position = (x + 1, y);
                        self.direction = Direction::Still;
                        self.previous_spot = (x, y);
                    }
                }
            }
            Direction::North => {
                if buffer.get(x as isize, y as isize - 1) != None
                    && buffer[(x, y - 1)] != self.maze_config.wall_color
                {
                    if (x, y - 1) == self.end_point {
                        println!("Congrats, you've finished the maze!");
                        self.position = (x, y - 1);
                        self.direction = Direction::Still;
                        self.previous_spot = (x, y);
                        self.game_over = true;
                    } else {
                        self.position = (x, y - 1);
                        self.direction = Direction::Still;
                        self.previous_spot = (x, y);
                    }
                }
            }
            Direction::South => {
                if buffer.get(x as isize, y as isize + 1) != None
                    && buffer[(x, y + 1)] != self.maze_config.wall_color
                {
                    if (x, y + 1) == self.end_point {
                        println!("Congrats, you've finished the maze!");
                        self.position = (x, y + 1);
                        self.direction = Direction::Still;
                        self.previous_spot = (x, y);
                        self.game_over = true;
                    } else {
                        self.position = (x, y + 1);
                        self.direction = Direction::Still;
                        self.previous_spot = (x, y);
                    }
                }
            }
            Direction::West => {
                if buffer.get(x as isize - 1, y as isize) != None
                    && buffer[(x - 1, y)] != self.maze_config.wall_color
                {
                    if (x - 1, y) == self.end_point {
                        println!("Congrats, you've finished the maze!");
                        self.position = (x - 1, y);
                        self.direction = Direction::Still;
                        self.previous_spot = (x, y);
                        self.game_over = true;
                    } else {
                        self.position = (x - 1, y);
                        self.direction = Direction::Still;
                        self.previous_spot = (x, y);
                    }
                }
            }
            Direction::Still => {
                self.position = self.position.clone();
                self.previous_spot = self.previous_spot;
            }
        }
    }
}

pub fn display(player: &Player, buffer: &mut WindowBuffer) {
    buffer[player.previous_spot] = player.maze_config.path_color;
    buffer[player.position] = player.player_color;
}

#[cfg(test)]
mod test {
    use rand::{rngs::StdRng, SeedableRng};

    use super::*;

    #[test]
    fn test_generate_maze() {
        let mut buffer: WindowBuffer = WindowBuffer::new(6, 6);
        let mut rng = StdRng::seed_from_u64(38);
        MazeConfig::default().generate(&mut buffer, &mut rng);

        insta::assert_snapshot!(buffer, @r###"
        #.....
        #.#.##
        #.#...
        #####.
        #.....
        ######
        "###);
    }

    #[test]
    fn generate_start_and_end() {
        let mut buffer: WindowBuffer = WindowBuffer::new(10, 10);
        let mut rng = StdRng::seed_from_u64(38);
        let mut player = Player::new(
            (0, 0),
            (0, 0),
            Direction::Still,
            (0, 0),
            MazeConfig::default(),
            false,
        );
        start_end_generator(&mut buffer, &mut rng, &mut player);

        insta::assert_snapshot!(buffer, @r###"
        ..........
        ..........
        #.........
        ..........
        ..........
        ..........
        ..........
        .........#
        ..........
        ..........
        "###);
    }

    #[test]
    fn direction_check() {
        let mut buffer: WindowBuffer = WindowBuffer::new(10, 10);
        let mut rng = StdRng::seed_from_u64(38);
        let mut player = Player::new(
            (0, 0),
            (0, 0),
            Direction::Still,
            (0, 0),
            MazeConfig::default(),
            false,
        );
        start_end_generator(&mut buffer, &mut rng, &mut player);

        insta::assert_snapshot!(buffer, @r###"
        ..........
        ..........
        #.........
        ..........
        ..........
        ..........
        ..........
        .........#
        ..........
        ..........
        "###);

        player.direction = Direction::North;
        player.direction(&buffer);
        display(&mut player, &mut buffer);

        insta::assert_snapshot!(buffer, @r###"
        ..........
        #.........
        #.........
        ..........
        ..........
        ..........
        ..........
        .........#
        ..........
        ..........
        "###);

        player.direction = Direction::North;
        player.direction(&buffer);
        display(&mut player, &mut buffer);

        insta::assert_snapshot!(buffer, @r###"
        #.........
        #.........
        #.........
        ..........
        ..........
        ..........
        ..........
        .........#
        ..........
        ..........
        "###);

        player.direction = Direction::East;
        player.direction(&buffer);
        display(&mut player, &mut buffer);

        insta::assert_snapshot!(buffer, @r###"
        ##........
        #.........
        #.........
        ..........
        ..........
        ..........
        ..........
        .........#
        ..........
        ..........
        "###);

        player.direction = Direction::East;
        player.direction(&buffer);
        display(&mut player, &mut buffer);

        insta::assert_snapshot!(buffer, @r###"
        ###.......
        #.........
        #.........
        ..........
        ..........
        ..........
        ..........
        .........#
        ..........
        ..........
        "###);

        player.direction = Direction::South;
        player.direction(&buffer);
        display(&mut player, &mut buffer);

        insta::assert_snapshot!(buffer, @r###"
        ###.......
        #.#.......
        #.........
        ..........
        ..........
        ..........
        ..........
        .........#
        ..........
        ..........
        "###);

        player.direction = Direction::South;
        player.direction(&buffer);
        display(&mut player, &mut buffer);

        insta::assert_snapshot!(buffer, @r###"
        ###.......
        #.#.......
        #.#.......
        ..........
        ..........
        ..........
        ..........
        .........#
        ..........
        ..........
        "###);

        player.direction = Direction::West;
        player.direction(&buffer);
        display(&mut player, &mut buffer);

        insta::assert_snapshot!(buffer, @r###"
        ###.......
        #.#.......
        ###.......
        ..........
        ..........
        ..........
        ..........
        .........#
        ..........
        ..........
        "###);

        player.direction = Direction::West;
        player.direction(&buffer);
        display(&mut player, &mut buffer);

        insta::assert_snapshot!(buffer, @r###"
        ###.......
        #.#.......
        ###.......
        ..........
        ..........
        ..........
        ..........
        .........#
        ..........
        ..........
        "###);
    }
}
