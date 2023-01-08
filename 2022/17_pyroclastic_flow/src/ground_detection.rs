type Position = (i32, i32);

#[derive(Clone, Copy)]
enum Direction {
    Left = 0b00,
    Up = 0b01,
    Right = 0b10,
    Down = 0b11,
}

impl Direction {
    fn is_vertical(&self) -> bool {
        (*self as usize) & 1 == 1
    }

    fn step(&self) -> i32 {
        if (*self as usize & 2) == 0 {
            -1
        } else {
            1
        }
    }

    fn rotate(&self, anticlockwise: bool) -> Direction {
        match (*self as usize + if anticlockwise { 3 } else { 1 }) % 4 {
            0 => Direction::Left,
            1 => Direction::Up,
            2 => Direction::Right,
            3 => Direction::Down,
            _ => unreachable!(),
        }
    }
}

struct Crawler {
    position: Position,
    direction: Direction,
}

/*
  ###
   #
## #    Example
 # #
 ###

 G/g: ground pixel (true/false)
 F/f: fore pixel  (true/false)
 arrow: crawler position and previous direction

 when ground is false (g) will be the next position

  ###                  ###               ###                     ###                     ###                   F##                    g###
  >F   rotate           #                 #   rotate              #   rotate              #                    ^G   rotate           f< #   rotate
##g#   clockwise     #Gv#  continue    ## #   anticlockwise    ## #   anticlockwise    ##f#  continue x 2    ## #   anticlockwise    ## #   clockwise
 # #                  #f#               Gv#                     #>F                     #^G                   # #                     # #
 ###                  ###               #F#                     #G#                     ###                   ###                     ###


 f                 ^g
 ^G##               ###
   #                 #   rotate
## #  continue    ## #   clockwise
 # #               # #
 ###               ###

*/

impl Crawler {
    fn get_x(&self) -> i32 {
        self.position.0
    }

    fn get_y(&self) -> i32 {
        self.position.1
    }

    fn rotate(&mut self, anticlockwise: bool) {
        self.direction = self.direction.rotate(anticlockwise);
    }

    fn adiacent_pixel(&self, direction: Direction) -> Position {
        if direction.is_vertical() {
            (self.position.0, self.position.1 - direction.step())
        } else {
            (self.position.0 + direction.step(), self.position.1)
        }
    }

    fn ground_pixel(&self) -> Position {
        self.adiacent_pixel(self.direction.rotate(false))
    }

    fn fore_pixel(&self) -> Position {
        self.adiacent_pixel(self.direction)
    }
}

pub fn detect_ground(top: i32, get_pixel_value: &dyn Fn(i32, i32) -> bool) -> Option<u64> {
    let get_pixel_value1 = |v: Position| -> bool {
        if v.0 > 6 || v.0 < 0 {
            true
        } else {
            get_pixel_value(v.0, v.1)
        }
    };
    let mut start_y = top;
    while start_y > 0 && !get_pixel_value(0, start_y - 1) {
        start_y -= 1;
    }
    if start_y == 0 {
        return None;
    }

    let mut crawler: Crawler = Crawler {
        direction: Direction::Right,
        position: (0, start_y),
    };
    let mut ground: u64 = 0;
    let mut segmets_count = 0;
    let mut segment_len0 = 0;
    let mut completed = false;

    while crawler.get_y() > 0 && crawler.get_x() < 7 && !completed {
        let mut segment_len: u32 = segment_len0;
        let mut anticlockwork_rotation: Option<bool> = None;

        segment_len0 = 0;
        while segment_len < 4 {
            let fore_pixel = crawler.fore_pixel();
            let fore_pixel_value = get_pixel_value1(fore_pixel);
            let ground_pixel = crawler.ground_pixel();
            let ground_pixel_value = get_pixel_value1(ground_pixel);
            if fore_pixel.0 == 7 && ground_pixel_value {
                completed = true;
                break;
            }
            if !ground_pixel_value || fore_pixel_value {
                anticlockwork_rotation = Some(ground_pixel_value);
                if !ground_pixel_value {
                    crawler.position = ground_pixel;
                    segment_len0 = 1;
                }
                break;
            }
            crawler.position = fore_pixel;
            segment_len += 1;
        }
        if crawler.get_y() == 0 {
            return None;
        }

        if segment_len > 0 {
            segmets_count += 1;
            if segmets_count > 15 {
                //no space in a u64
                return None;
            }
            ground |= (crawler.direction as u32 | ((segment_len - 1) << 2)) as u64;
            ground <<= 4;
        }
        if let Some(anticlockwork) = anticlockwork_rotation {
            crawler.rotate(anticlockwork);
        }
    }
    ground |= segmets_count;
    Some(ground)
}

//only for demo purposes
pub fn ground_to_string(ground: u64) -> String {
    let mut segments_count = ground & 0b1111;
    let mut bits = ground;
    let mut ret = String::new();
    while segments_count > 0 {
        bits >>= 4;
        segments_count -= 1;

        let direction = bits & 0b0011;
        let segments_len = ((bits & 0b1100) >> 2) + 1;
        for _ in 0..segments_len {
            ret.push(match direction {
                0 => '<',
                1 => '^',
                2 => '>',
                3 => 'v',
                _ => unreachable!(),
            });
        }
    }
    ret.chars().rev().collect()
}
