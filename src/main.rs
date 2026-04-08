mod render;

use render::*;

use braille::{BrailleCharUnOrdered, BrailleCharGridVector};

use std::io::{stdout, Write, Result};
use std::time::{Duration, Instant};
use std::ops::{Add, Mul};

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, DisableLineWrap, Clear, ClearType, size, enable_raw_mode},
    cursor::{MoveTo, Hide, Show},
    style::Print,
    event::{Event, KeyEvent, KeyCode, read, poll}
};
use glam::Vec3;
use rand::RngExt;

fn get_screen_size() -> Result<(usize, usize)> {
    let (w, h) = size()?;

    return Ok((w as usize, h as usize));
}

#[inline(always)]
fn index<N: Add<Output = N> + Mul<Output = N>>(x: N, y: N, width: N) -> N {
    return x + y * width;
}

fn main() -> Result<()> {
    let mut stdout = stdout();

    let mut rng = rand::rng();

    let (mut prev_cols, mut prev_rows) = get_screen_size()?;
    let (mut cols, mut rows) = get_screen_size()?;

    execute!(stdout, EnterAlternateScreen)?;
    execute!(stdout, Clear(ClearType::All))?;
    execute!(stdout, DisableLineWrap)?;

    let mut grid: BrailleCharGridVector<BrailleCharUnOrdered> = BrailleCharGridVector::new(cols, rows);

    // let mut img = vec![Vec3::ZERO; cols * rows * 8];
    let mut img = Canva::new(cols * 2, rows * 4);

    // let scene1 = Scene3D {
    //     camera: Camera {
    //         pos: Vec3::ZERO,
    //         yaw: 0.0,
    //         pitch: 0.0,
    //         roll: 0.0
    //     },
    //     vertices: vec![
    //         Vec3::new(0.8, 1.0, 1.0),
    //         Vec3::new(0.8, 1.0, 1.8),
    //         Vec3::new(-0.8, 1.0, 1.8),
    //         Vec3::new(0.8, 0.0, 1.8)
    //     ],
    //     faces: vec![
    //         Face {
    //             vertices: (0, 1, 2),
    //             normal: Vec3::new(0.0, 1.0, 0.0),
    //             color: 255.0
    //         },
    //         Face {
    //             vertices: (1, 2, 3),
    //             normal: Vec3::new(0.0, 0.0, -1.0),
    //             color: 255.0
    //         }
    //     ],
    //     lights: vec![
    //         Light {
    //             pos: Vec3::new(0.0, 3.2, 0.0),
    //             intensity: 4.0
    //         }
    //     ]
    // };
    let mut scene = Scene3D {
        camera: Camera {
            pos: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0
        },
        vertices: vec![
            Vec3::new(0.0, 0.707, 1.0),
            Vec3::new(0.707, 0.707, 1.707),
            Vec3::new(0.0, -0.707, 1.707),
            Vec3::new(-0.707, 0.707, 1.707),
            Vec3::new(0.0, 0.0, 3.0), // 4
            Vec3::new(5.0, -5.0, 2.0),
            Vec3::new(-5.0, -5.0, 2.0),
            Vec3::new(0.0, 5.0, 2.0)
        ],
        faces: vec![
            Face {
                vertices: (0, 1, 2),
                normal: Vec3::new(1.0, 1.0, -1.0).normalize(),
                color: 255.0
            },
            Face {
                vertices: (0, 2, 3),
                normal: Vec3::new(-1.0, 1.0, -1.0).normalize(),
                color: 255.0
            },
            Face {
                vertices: (4, 5, 6),
                normal: Vec3::new(-1.0, -1.0, -1.0).normalize(),
                color: 200.0
            },
            Face {
                vertices: (4, 6, 7),
                normal: Vec3::new(1.0, -1.0, -1.0).normalize(),
                color: 200.0
            },
            Face {
                vertices: (4, 7, 5),
                normal: Vec3::new(0.0, -1.0, -1.0).normalize(),
                color: 200.0
            },
        ],
        lights: vec![
            Light {
                pos: Vec3::new(0.0, 0.0, 0.0),
                intensity: 5.0
            }
        ]
    };

    stdout.flush()?;

    enable_raw_mode()?;
    execute!(stdout, Hide)?;

    let mut last_frame = Instant::now();

    let mut time = 0.0_f32;
    loop {
        time += 0.01;

        scene.lights[0].pos.x = time.cos();
        scene.lights[0].pos.y = time.sin();

        let now = Instant::now();
        let dt = now.duration_since(last_frame);

        (prev_cols, prev_rows) = (cols, rows);
        (cols, rows) = get_screen_size()?;

        if cols == 0 || rows == 0 {
            continue;
        }

        if poll(Duration::from_millis(5))? {
            if let Event::Key(KeyEvent { code, .. }) = read()? {
                match code {
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
        }

        grid.resize(cols, rows, (0, 0), BrailleCharUnOrdered::EMPTY);
        img.clear();
        // img.resize(cols * rows * 8, Vec3::ZERO);
        img.resize(cols * 2, rows * 4);

        // render
        // for y in 0..(rows*4) {
        //     for x in 0..(cols*2) {
        //         let d2 = ((x as isize - cols as isize)).pow(2) / (cols as isize / 4).max(1) + ((y as isize - rows as isize * 2)).pow(2) / (rows as isize / 2).max(1);
        //         let l = 255.0 - ((d2 * 2) as f32 * (1.0 - 0.8 * (1.0 - time.cos().abs())) ).min(255.0);
        //         img.array[x + y * cols * 2] = l;
        //     }
        // }
        // for _ in 0..10 {
        //     let x = rng.random_range(0..(cols*2));
        //     let y = rng.random_range(0..(rows*4));
        //     img.draw_circle(x, y, 10, 255.0);
        // }

        scene.render(&mut img);

        // dithering
        for y in 0..(rows*4) {
            for x in 0..(cols*2) {
                let oldpixel = img.array[index(x, y, cols * 2)].clamp(0.0, 255.0);
                let (b, nl) = match oldpixel {
                    0.0..127.0 => (false, 0.0),
                    _ => (true, 255.0)
                };
                let newpixel = nl;

                unsafe { grid.set_unchecked(x, y, b) };

                let mut quant_error = oldpixel - newpixel;

                quant_error /= 8.0;

                let right = x + 1 < cols * 2;
                let right2 = x + 2 < cols * 2;
                let left = x > 0;
                let down = y + 1 < rows * 4;
                let down2 = y + 2 < rows * 4;

                if right {
                    img.array[index(x+1, y, cols * 2)] += quant_error;
                    if right2 {
                        img.array[index(x+2, y, cols * 2)] += quant_error;
                    }
                    if down {
                        img.array[index(x+1, y+1, cols * 2)] += quant_error;
                    }
                }
                if down {
                    img.array[index(x, y+1, cols * 2)] += quant_error;
                    if left {
                        img.array[index(x-1, y+1, cols * 2)] += quant_error;
                    }
                    if down2 {
                        img.array[index(x, y+2, cols * 2)] += quant_error;
                    }
                }
            }
        }

        let ms = dt.as_secs_f64() * 1000.0;
        let fps = 1.0 / dt.as_secs_f64();

        let info = format!("{:.2} ms | {:.0} FPS", ms, fps);

        let w = info.len();
        let mx = cols.saturating_sub(w);

        if rows > 1 { // line 0
            let mut line = String::with_capacity(cols as usize);

            for x in 0..mx.saturating_sub(3) {
                line.push(grid[(x as usize, 0)].char());
            }
            let mut char = grid[(mx.saturating_sub(3), 0)];
            char &= 0b_1111_1110;
            line.push(char.char());
            for x in mx.saturating_sub(2)..(cols-2) {
                let mut char = grid[(x, 0)];
                char &= 0b_1111_1100;
                line.push(char.char());
            }
            let mut char = grid[(cols-1, 0)];
            char &= 0b_1111_1101;
            line.push(char.char());
            line.push(grid[(cols, 0)].char());

            execute!(
                stdout,
                MoveTo(0, 0),
                Print(&line)
            )?;
        }

        if rows > 2 { // line 1
            let mut line = String::with_capacity(cols as usize);

            for x in 0..mx.saturating_sub(3) {
                line.push(grid[(x as usize, 1)].char());
            }
            let mut char = grid[(mx.saturating_sub(3), 1)];
            char &= 0b_1010_1010;
            line.push(char.char());
            line.push_str(&info);
            let mut char = grid[(cols-1, 1)];
            char &= 0b_0101_0101;
            line.push(char.char());
            line.push(grid[(cols, 1)].char());

            execute!(
                stdout,
                MoveTo(0, 1),
                Print(&line)
            )?;
        }

        if rows > 3 { // line 2
            let mut line = String::with_capacity(cols as usize);

            for x in 0..mx.saturating_sub(3) {
                line.push(grid[(x as usize, 2)].char());
            }
            let mut char = grid[(mx.saturating_sub(3), 2)];
            char &= 0b_1011_1111;
            line.push(char.char());
            for x in mx.saturating_sub(2)..(cols-2) {
                let mut char = grid[(x, 2)];
                char &= 0b_0011_1111;
                line.push(char.char());
            }
            let mut char = grid[(cols-1, 2)];
            char &= 0b_0111_1111;
            line.push(char.char());
            line.push(grid[(cols, 2)].char());

            execute!(
                stdout,
                MoveTo(0, 2),
                Print(&line)
            )?;
        }

        if rows > 3 {
            for y in 3..rows {
                let mut line = String::with_capacity(cols as usize);

                for x in 0..cols {
                    line.push(grid[(x as usize, y as usize)].char());
                }

                execute!(
                    stdout,
                    MoveTo(0, y as u16),
                    Print(&line)
                )?;
            }
        }

        stdout.flush()?;
        last_frame = now;
    }

    execute!(stdout, LeaveAlternateScreen)?;

    return Ok(());
}

