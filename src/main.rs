mod render;

use render::*;

use braille::{BrailleCharUnOrdered, BrailleCharGridVector};

use std::io::{stdout, Write, Result};
use std::time::{Duration, Instant};
use std::ops::{Add, Mul};

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, DisableLineWrap, Clear, ClearType, size, enable_raw_mode},
    cursor::{MoveTo, Hide},
    style::Print,
    event::{Event, KeyEvent, KeyCode, ModifierKeyCode, read, poll}
};
use glam::{Vec3, Vec2};
use image;

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

    let (mut cols, mut rows) = get_screen_size()?;

    execute!(stdout, EnterAlternateScreen)?;
    execute!(stdout, Clear(ClearType::All))?;
    execute!(stdout, DisableLineWrap)?;

    let mut grid: BrailleCharGridVector<BrailleCharUnOrdered> = BrailleCharGridVector::new(cols, rows);

    let mut img = Canva::new(cols * 2, rows * 4);

    let mut scene = Scene3D {
        camera: Camera::default(),
        vertices: vec![
            Vec3::new(0.0, 0.707, 1.0),
            Vec3::new(0.707, 0.707, 1.707),
            Vec3::new(0.0, -0.707, 1.707),
            Vec3::new(-0.707, 0.707, 1.707),
            Vec3::new(2.0, 0.707, 0.1), // 4
            Vec3::new(2.0, 0.707, 5.0),
            Vec3::new(-2.0, 0.707, 5.0),
            Vec3::new(-2.0, 0.707, 0.1)
        ],
        uv: vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0),
        ],
        faces: vec![],
        lights: vec![
            Light {
                pos: Vec3::new(0.0, 0.0, 0.0),
                intensity: 3.0
            },
            Light {
                pos: Vec3::ZERO,
                intensity:  0.0
            }
        ],
        textures: vec![image::open("./bunny.jpg").unwrap().into_luma8()]
    };
    // new_face_from_index(&mut scene, (0, 1, 2), 0, (3, 3, 3));
    // new_face_from_index(&mut scene, (0, 2, 3), 0, (3, 3, 3));
    new_face_from_index(&mut scene, (4, 5, 6), 0, (3, 1, 0));
    new_face_from_index(&mut scene, (4, 6, 7), 0, (3, 0, 2));

    stdout.flush()?;

    enable_raw_mode()?;
    execute!(stdout, Hide)?;

    let mut last_frame = Instant::now();

    loop {
        let now = Instant::now();
        let dt = now.duration_since(last_frame);

        (cols, rows) = get_screen_size()?;

        if cols == 0 || rows == 0 {
            continue;
        }

        let cam_forward = scene.camera.forward();
        let cam_right = scene.camera.right();
        let cam_up = -Vec3::Y;
        if poll(Duration::from_millis(5))? {
            if let Event::Key(KeyEvent { code, .. }) = read()? {
                match code {
                    KeyCode::Esc => break,
                    KeyCode::Char('w') => scene.camera.pos += cam_forward * 0.02,
                    KeyCode::Char('s') => scene.camera.pos -= cam_forward * 0.02,
                    KeyCode::Char('d') => scene.camera.pos += cam_right * 0.02,
                    KeyCode::Char('a') => scene.camera.pos -= cam_right * 0.02,
                    KeyCode::Char(' ') => scene.camera.pos += cam_up * 0.05,
                    KeyCode::Char('v') => scene.camera.pos -= cam_up * 0.05,
                    KeyCode::Modifier(ModifierKeyCode::LeftControl) => scene.camera.pos -= cam_up * 0.05,
                    KeyCode::Char('e') => scene.camera.yaw -= 0.02,
                    KeyCode::Char('q') => scene.camera.yaw += 0.02,
                    KeyCode::Char('r') => scene.camera.pitch -= 0.02,
                    KeyCode::Char('f') => scene.camera.pitch += 0.02,
                    KeyCode::Char('g') => scene.camera.roll -= 0.02,
                    KeyCode::Char('t') => scene.camera.roll += 0.02,
                    KeyCode::Char('y') => scene.camera.fov += 0.02,
                    KeyCode::Char('h') => scene.camera.fov -= 0.02,
                    _ => {}
                }
            }
        }
        scene.lights[1].pos = scene.camera.pos;

        grid.resize(cols, rows, (0, 0), BrailleCharUnOrdered::EMPTY);
        img.clear();
        img.resize(cols * 2, rows * 4);

        // render
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

        let info = format!("{:>5.2} ms | {:>3.0} FPS", ms, fps);

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

