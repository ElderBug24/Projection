mod render;

use render::*;

use braille::{BrailleCharUnOrdered, BrailleCharGridVector};

use std::io::{stdout, Write, Result};
use std::time::{Duration, Instant};
use std::ops::{Add, Mul};
use std::fmt::Write as Write_;

use crossterm::{
    execute,
    queue,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, DisableLineWrap, Clear, ClearType, size, enable_raw_mode},
    cursor::{MoveTo, Hide},
    style::Print,
    event::{Event, KeyEvent, KeyCode, read, poll}
};
use glam::{Vec3, Vec2};
use image::{self, Rgb};

fn get_screen_size() -> Result<(usize, usize)> {
    let (w, h) = size()?;

    return Ok((w as usize, h as usize));
}

#[inline(always)]
fn index<N: Add<Output = N> + Mul<Output = N>>(x: N, y: N, width: N) -> N {
    return x + y * width;
}

fn main() -> Result<()> {
    let mut display_color = false;
    let mut stdout = stdout();

    let (mut cols, mut rows) = get_screen_size()?;
    let mut grid: BrailleCharGridVector<BrailleCharUnOrdered> = BrailleCharGridVector::new(cols, rows);
    let mut canva = Canva::new(cols * 2, rows * 4);
    let mut scene = Scene3D {
        camera: Camera::default(),
        lights: vec![
            Light {
                pos: Vec3::ZERO,
                intensity: 30.0
            },
            Light {
                pos: Vec3::ZERO,
                intensity:  0.0
            }
        ],
        buffered_faces: vec![],
        buffered_textures: vec![]
    };
    let mut bunny = Model3D {
        vertices: vec![
            Vec3::new(2.0, -2.0, 10.0),
            Vec3::new(2.0, 3.0, 10.0),
            Vec3::new(-2.0, 3.0, 10.0),
            Vec3::new(-2.0, -2.0, 10.0)
        ],
        uv: vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0),
        ],
        normals: vec![],
        faces: vec![],
        texture: image::open("./bunny.jpg").unwrap().into_rgb8().into()

    };
    new_face_from_index(&mut bunny, (0, 1, 2), (3, 1, 0));
    new_face_from_index(&mut bunny, (0, 2, 3), (3, 0, 2));
    let mut cat = Model3D {
        vertices: vec![
            Vec3::new(2.0, -2.0, 5.0),
            Vec3::new(2.0, 3.0, 5.0),
            Vec3::new(2.0, 3.0, 10.0),
            Vec3::new(2.0, -2.0, 10.0)
        ],
        uv: vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0),
        ],
        normals: vec![],
        faces: vec![],
        texture: image::open("./cat.jpg").unwrap().into_rgb8().into()

    };
    new_face_from_index(&mut cat, (0, 1, 2), (3, 1, 0));
    new_face_from_index(&mut cat, (0, 2, 3), (3, 0, 2));

    queue!(stdout, EnterAlternateScreen)?;
    queue!(stdout, Clear(ClearType::All))?;
    execute!(stdout, DisableLineWrap)?;
    queue!(stdout, Hide)?;
    enable_raw_mode()?;
    stdout.flush()?;

    let mut last_frame = Instant::now();
    let mut line_buf = String::new();

    loop {
        let now = Instant::now();
        let dt = now.duration_since(last_frame);

        (cols, rows) = get_screen_size()?;

        if cols == 0 || rows == 0 {
            continue;
        }

        let cam_forward = scene.camera.forward();
        let cam_right = scene.camera.right();
        let cam_up = Vec3::Y;
        if poll(Duration::ZERO)? {
            if let Event::Key(KeyEvent { code, .. }) = read()? {
                match code {
                    KeyCode::Esc => break,
                    KeyCode::Char('w') => scene.camera.pos += cam_forward * 0.05, // forward
                    KeyCode::Char('s') => scene.camera.pos -= cam_forward * 0.05, // backward
                    KeyCode::Char('d') => scene.camera.pos += cam_right * 0.05,   // right
                    KeyCode::Char('a') => scene.camera.pos -= cam_right * 0.05,   // left
                    KeyCode::Char(' ') => scene.camera.pos += cam_up * 0.05,      // up
                    KeyCode::Char('v') => scene.camera.pos -= cam_up * 0.05,      // down
                    KeyCode::Char('e') => scene.camera.yaw -= 0.03,
                    KeyCode::Char('q') => scene.camera.yaw += 0.03,
                    KeyCode::Char('r') => scene.camera.pitch += 0.03,
                    KeyCode::Char('f') => scene.camera.pitch -= 0.03,
                    KeyCode::Char('g') => scene.camera.roll += 0.03,
                    KeyCode::Char('t') => scene.camera.roll -= 0.03,
                    KeyCode::Char('y') => scene.camera.fov += 0.03,
                    KeyCode::Char('h') => scene.camera.fov -= 0.03,
                    KeyCode::Char('x') => display_color = false,
                    KeyCode::Char('c') => display_color = true,
                    _ => {}
                }
            }
        }
        scene.lights[1].pos = scene.camera.pos;

        grid.resize(cols, rows, (0, 0), BrailleCharUnOrdered::EMPTY);
        canva.clear();
        canva.resize(cols * 2, rows * 4);

        // render
        scene.clear_queue();
        scene.queue_render(&bunny);
        scene.queue_render(&cat);
        scene.render(&mut canva);

        // dithering
        for y in 0..(rows*4) {
            for x in 0..(cols*2) {
                let oldpixel = canva.array[index(x, y, cols * 2)].clamp(Vec3::ZERO, Vec3::ONE);
                let (b, nl) = match oldpixel.element_sum() {
                    0.0..1.5 => (false, 0.0),
                    _ => (true, 1.0)
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
                    canva.array[index(x+1, y, cols * 2)] += quant_error;
                    if right2 {
                        canva.array[index(x+2, y, cols * 2)] += quant_error;
                    }
                    if down {
                        canva.array[index(x+1, y+1, cols * 2)] += quant_error;
                    }
                }
                if down {
                    canva.array[index(x, y+1, cols * 2)] += quant_error;
                    if left {
                        canva.array[index(x-1, y+1, cols * 2)] += quant_error;
                    }
                    if down2 {
                        canva.array[index(x, y+2, cols * 2)] += quant_error;
                    }
                }
            }
        }

        let ms = dt.as_secs_f64() * 1000.0;
        let fps = 1.0 / dt.as_secs_f64();

        let info = format!("{:>5.2} ms | {:>3.0} FPS", ms, fps);

        let w = info.len();
        // let offset = (cols - w) / 2;
        let offset = 0;
        let mx = cols.saturating_sub(w+offset);

        queue!(
            stdout,
            MoveTo(0, 0),
            Print("\x1b[39m")
        )?;

        line_buf.reserve_exact(cols as usize);
        if rows > 1 { // line 0
            line_buf.clear();
            let mut x = 0;

            for _ in 0..mx.saturating_sub(3) {
                if display_color {
                    let Rgb([r, g, b]) = vec3_to_rgb(canva.average_color(x, 0));
                    write!(line_buf, "\x1b[38;2;{};{};{}m", r, g, b).unwrap();
                }
                line_buf.push(grid[(x as usize, 0)].char());
                x += 1;
            }
            if display_color {
                let Rgb([r, g, b]) = vec3_to_rgb(canva.average_color(x, 0));
                write!(line_buf, "\x1b[38;2;{};{};{}m", r, g, b).unwrap();
            }
            line_buf.push((grid[(x, 0)] & 0b_1111_1110).char());
            x += 1;
            for _ in 0..w {
                if display_color {
                    let Rgb([r, g, b]) = vec3_to_rgb(canva.average_color(x, 0));
                    write!(line_buf, "\x1b[38;2;{};{};{}m", r, g, b).unwrap();
                }
                line_buf.push((grid[(x, 0)] & 0b_1111_1100).char());
                x += 1;
            }
            if display_color {
                let Rgb([r, g, b]) = vec3_to_rgb(canva.average_color(x, 0));
                write!(line_buf, "\x1b[38;2;{};{};{}m", r, g, b).unwrap();
            }
            line_buf.push((grid[(x, 0)] & 0b_1111_1101).char());
            x += 1;
            while x < cols {
                if display_color {
                    let Rgb([r, g, b]) = vec3_to_rgb(canva.average_color(x, 0));
                    write!(line_buf, "\x1b[38;2;{};{};{}m", r, g, b).unwrap();
                }
                line_buf.push(grid[(x, 0)].char());
                x += 1;
            }

            queue!(
                stdout,
                MoveTo(0, 0),
                Print(&line_buf)
            )?;
        }

        if rows > 2 { // line 1
            line_buf.clear();
            let mut x = 0;

            for _ in 0..mx.saturating_sub(3) {
                if display_color {
                    let Rgb([r, g, b]) = vec3_to_rgb(canva.average_color(x, 1));
                    write!(line_buf, "\x1b[38;2;{};{};{}m", r, g, b).unwrap();
                }
                line_buf.push(grid[(x as usize, 1)].char());
                x += 1;
            }
            if display_color {
                let Rgb([r, g, b]) = vec3_to_rgb(canva.average_color(x, 1));
                write!(line_buf, "\x1b[38;2;{};{};{}m", r, g, b).unwrap();
            }
            line_buf.push((grid[(x, 1)] & 0b_1010_1010).char());
            x += 1;
            if display_color { line_buf.push_str("\x1b[0m"); }
            line_buf.push_str(&info);
            x += w;
            if display_color {
                let Rgb([r, g, b]) = vec3_to_rgb(canva.average_color(x, 1));
                write!(line_buf, "\x1b[38;2;{};{};{}m", r, g, b).unwrap();
            }
            line_buf.push((grid[(cols-1, 1)] & 0b_0101_0101).char());
            x += 1;
            while x < cols {
                if display_color {
                    let Rgb([r, g, b]) = vec3_to_rgb(canva.average_color(x, 1));
                    write!(line_buf, "\x1b[38;2;{};{};{}m", r, g, b).unwrap();
                }
                line_buf.push(grid[(x, 1)].char());
                x += 1;
            }

            queue!(
                stdout,
                MoveTo(0, 1),
                Print(&line_buf)
            )?;
        }

        if rows > 3 { // line 2
            line_buf.clear();
            let mut x = 0;

            for _ in 0..mx.saturating_sub(3) {
                if display_color {
                    let Rgb([r, g, b]) = vec3_to_rgb(canva.average_color(x, 2));
                    write!(line_buf, "\x1b[38;2;{};{};{}m", r, g, b).unwrap();
                }
                line_buf.push(grid[(x as usize, 2)].char());
                x += 1;
            }
            if display_color {
                let Rgb([r, g, b]) = vec3_to_rgb(canva.average_color(x, 2));
                write!(line_buf, "\x1b[38;2;{};{};{}m", r, g, b).unwrap();
            }
            line_buf.push((grid[(mx.saturating_sub(3), 2)] & 0b_1011_1111).char());
            x += 1;
            for _ in 0..w {
                if display_color {
                    let Rgb([r, g, b]) = vec3_to_rgb(canva.average_color(x, 2));
                    write!(line_buf, "\x1b[38;2;{};{};{}m", r, g, b).unwrap();
                }
                line_buf.push((grid[(x, 2)] & 0b_0011_1111).char());
                x += 1;
            }
            if display_color {
                let Rgb([r, g, b]) = vec3_to_rgb(canva.average_color(x, 2));
                write!(line_buf, "\x1b[38;2;{};{};{}m", r, g, b).unwrap();
            }
            line_buf.push((grid[(x, 2)] & 0b_0111_1111).char());
            x += 1;
            while x < cols {
                if display_color {
                    let Rgb([r, g, b]) = vec3_to_rgb(canva.average_color(x, 2));
                    write!(line_buf, "\x1b[38;2;{};{};{}m", r, g, b).unwrap();
                }
                line_buf.push(grid[(x, 2)].char());
                x += 1;
            }

            queue!(
                stdout,
                MoveTo(0, 2),
                Print(&line_buf)
            )?;
        }

        if rows > 3 {
            for y in 3..rows {
                line_buf.clear();

                for x in 0..cols {
                    if display_color {
                        let Rgb([r, g, b]) = vec3_to_rgb(canva.average_color(x, y));
                        write!(line_buf, "\x1b[38;2;{};{};{}m", r, g, b).unwrap();
                    }
                    line_buf.push(grid[(x as usize, y as usize)].char());
                }

                queue!(
                    stdout,
                    MoveTo(0, y as u16),
                    Print(&line_buf)
                )?;
            }
        }

        stdout.flush()?;
        last_frame = now;
    }

    execute!(stdout, LeaveAlternateScreen)?;

    return Ok(());
}

