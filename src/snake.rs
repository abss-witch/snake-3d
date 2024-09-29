use std::f32::consts::PI;
use thin_engine::{prelude::*, glium_types::vectors::*};
use rand::{rngs::ThreadRng, Rng};
pub struct Snake{
    pub pos: IVec3,
    pub direction: Direction
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction{
    Forward,
    Back,
    Left,
    Right,
    Up,
    Down
}
impl Direction{
    pub fn dir(&self) -> IVec3 {
        match self {
            Direction::Forward =>  IVec3::Y,
            Direction::Back    => -IVec3::Y,
            Direction::Left    => -IVec3::X,
            Direction::Right   =>  IVec3::X,
            Direction::Up      =>  IVec3::Z,
            Direction::Down    => -IVec3::Z
        }
    }
    pub fn rot(&self) -> Quaternion{
        match self {
            Direction::Forward => Quaternion::from_y_rot( PI      ),
            Direction::Back    => Quaternion::from_y_rot( 0.0     ),
            Direction::Left    => Quaternion::from_y_rot( PI / 2.0) ,
            Direction::Right   => Quaternion::from_y_rot(-PI / 2.0),
            Direction::Up      => Quaternion::from_x_rot( PI / 2.0),
            Direction::Down    => Quaternion::from_x_rot(-PI / 2.0),
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum State {
    Wait,
    Alive,
    Dead,
    Win
}
pub struct Board {
    pub snake: Snake,
    pub snake_body: Vec<IVec3>,
    pub apple_pos: IVec3,
    width: usize,
    height: usize,
    depth: usize,
    grid: Vec<Vec<Vec<Point>>>,
    pub state: State
}
impl Board {
    pub fn board_size(&self) -> IVec3 {
        ivec3(self.width as i32, self.height as i32, self.depth as i32)
    }
    pub fn update(&mut self, rng: &mut ThreadRng) {
        let new_pos = self.snake.pos + self.snake.direction.dir();
        let collected_apple = self.apple_pos == new_pos;
        
        if !collected_apple {
            let pos = self.snake_body.remove(0);
            *self.point_at(pos).unwrap() = Point::Empty;
        }

        let point = self.point_at(new_pos);
        if point.is_none() || point.is_some_and(|i| *i == Point::Snake) {
            self.state = State::Dead;
            return;
        }

        self.snake_body.push(new_pos);
        *self.point_at(new_pos).unwrap() = Point::Snake;
        self.snake.pos = new_pos;

        // check win
        if self.snake_body.len() == self.width*self.height*self.depth {
            self.state = State::Win;
            return;
        }

        if collected_apple { self.spawn_apple(rng) }
    }
    pub fn board_to_space(&self, value: IVec3) -> Vec3 {
        let IVec3 { x, y, z } = value;
        let IVec3 { x: sx, y: sy, z: sz } = self.board_size() - IVec3::ONE;
        let x_offset = sx as f32 / 2.0;
        let y_offset = sy as f32 / 2.0;
        let z_offset = sz as f32 / 2.0;
        vec3(x as f32 - x_offset, z as f32 - z_offset, y as f32 - y_offset)
    }
    pub fn shadow_matrix(&self, value: IVec3) -> Mat4 {
        let IVec3 { x, y, z: s } = value;
        let x = x as f32 - (self.width  - 1) as f32 / 2.0;
        let y = y as f32 - (self.height - 1) as f32 / 2.0;
        let z = -0.5 - (self.depth  - 1) as f32 / 2.0;
        Mat4::from_transform(
            vec3(x, z, y), Vec3::splat(0.5 - (f32::sqrt(s as f32)*0.05).min(0.5)),
            Quaternion::from_x_rot(PI/2.0)
        )
    }
    fn spawn_apple(&mut self, rng: &mut ThreadRng){
        let mut i: usize = rng.gen_range(0..=usize::MAX);
        for _ in 0..(self.width*self.height*self.depth) {
            let z = i % self.depth;
            let y = i / self.depth % self.height;
            let x = i / (self.depth*self.height) % self.width;
            let pos = ivec3(x as i32, y as i32, z as i32);

            match &self.grid[x][y][z] {
                Point::Empty => { self.apple_pos = pos; break; },
                _ => i = i.wrapping_add(1)
            }
        }
    }
    pub fn new(width: usize, depth: usize, height: usize) -> Self{
        let mut grid = vec![vec![vec![Point::Empty; height]; depth]; width];
        grid[0][0][0] = Point::Snake;
        Self { 
            snake: Snake {
                pos: IVec3::ZERO,
                direction: Direction::Forward
            },
            apple_pos: ivec3(0, 1, 0), snake_body: vec![IVec3::ZERO],
            width, height, depth, grid,
            state: State::Alive
        }
    }
    pub fn point_at(&mut self, index: IVec3) -> Option<&mut Point> {
        self.grid.get_mut(index.x as usize)
            .and_then(|i| i.get_mut(index.y as usize))
            .and_then(|i| i.get_mut(index.z as usize))
    }
    pub fn matrices(&self) -> (Mat4, Vec<Mat4>, Vec<IVec3>) {
        let mut shadows = vec![vec![None; self.width]; self.depth];
        let mut snake = vec![];
        let mut shadow = vec![self.apple_pos.truncate()];
        shadows[self.apple_pos.x as usize][self.apple_pos.y as usize] = Some(self.apple_pos.z);
        for i in &self.snake_body {
            snake.push(Mat4::from_pos(self.board_to_space(*i)));
            let ref_height: &mut Option<i32> = &mut shadows[i.x as usize][i.y as usize];
            if let Some(height) = ref_height {
                *ref_height = Some((*height).min(i.z));
            } else {
                shadow.push(i.truncate());
                *ref_height = Some(i.z);
            }
        }

        let apple_mat = Mat4::from_pos(self.board_to_space(self.apple_pos));
        (apple_mat, snake, shadow.into_iter().map(|i| {
            let height = shadows[i.x as usize][i.y as usize];
            i.extend(height.unwrap())
        }).collect())
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Point{
    Empty,
    Snake
}
