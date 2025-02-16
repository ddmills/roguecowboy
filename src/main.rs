use macroquad::prelude::*;

const CRT_FRAGMENT_SHADER: &str = include_str!("assets/crt-shader.glsl");
const CRT_VERTEX_SHADER:&str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;

varying lowp vec2 uv;
varying lowp vec4 color;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}
";


enum GameState {
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Rogue Cowboy".to_string(),
        window_width: 800,
        window_height: 600,
        // high_dpi: todo!(),
        fullscreen: false,
        // sample_count: todo!(),
        window_resizable: true,
        // platform: todo!(),
        ..Default::default()
    }
}

fn get_preferred_size(texel_size: u32) -> IVec2 {
    ivec2((screen_width() / texel_size as f32) as i32, (screen_height() / texel_size as f32) as i32)
}

#[macroquad::main(window_conf)]
async fn main() {
    set_default_filter_mode(FilterMode::Nearest);
    let texel_size = 1;
    let mut pref_size: IVec2 = get_preferred_size(texel_size);

    let mut main_render_target = render_target(pref_size.x as u32, pref_size.y as u32);
    main_render_target.texture.set_filter(FilterMode::Nearest);

    rand::srand(miniquad::date::now() as u64);

    let mut game_state = GameState::MainMenu;

    let crt_material = load_material(
        ShaderSource::Glsl {
            vertex: CRT_VERTEX_SHADER,
            fragment: CRT_FRAGMENT_SHADER,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("iResolution", UniformType::Float2),
                UniformDesc::new("iTime", UniformType::Float1),
            ],
            ..Default::default()
        },
    )
    .unwrap();

    loop {
        pref_size = get_preferred_size(texel_size);
        let pref_size_f32 = pref_size.as_vec2();
        let cur_target_size = main_render_target.texture.size().as_ivec2();

        if cur_target_size != pref_size {
            main_render_target = render_target(pref_size.x as u32, pref_size.y as u32);
            main_render_target.texture.set_filter(FilterMode::Nearest);
        }

        set_camera(&Camera2D {
            zoom: vec2(1. / pref_size_f32.x * 2., 1. / pref_size_f32.y * 2.),
            target: vec2((pref_size_f32.x * 0.5f32).floor(), (pref_size_f32.y * 0.5f32).floor()),
            render_target: Some(main_render_target.clone()),
            ..Default::default()
        });
        clear_background(BLACK);
        gl_use_default_material();

        match game_state {
            GameState::MainMenu => {
                if is_key_pressed(KeyCode::Escape) {
                    std::process::exit(0);
                }
                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Playing;
                }
                let text = "Press space";
                let text_dimensions = measure_text(text, None, 32, 1.0);
                draw_text_ex(
                    text,
                    pref_size_f32.x / 2.0 - text_dimensions.width / 2.0,
                    pref_size_f32.y / 2.0,
                    TextParams {
                        font: None,
                        font_size: 32, font_scale: 1.0, font_scale_aspect: 1.0, rotation: 0., color: WHITE }
                );
            }
            GameState::Playing => {
                let delta_time = get_frame_time();
                if is_key_pressed(KeyCode::Escape) {
                    game_state = GameState::Paused;
                }

                draw_circle(50., 75., 200., YELLOW);
                draw_text(
                    format!("yooo: {}", delta_time).as_str(),
                    32.0,
                    32.,
                    16.0,
                    WHITE,
                );
            }
            GameState::Paused => {
                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Playing;
                }
                let text = "Paused";
                let text_dimensions = measure_text(text, None, 32, 1.0);
                draw_text(
                    text,
                    pref_size_f32.x / 2.0 - text_dimensions.width / 2.0,
                    pref_size_f32.y / 2.0,
                    32.0,
                    WHITE,
                );
            }
            GameState::GameOver => {
                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::MainMenu;
                }
                let text = "GAME OVER!";
                let text_dimensions = measure_text(text, None, 16, 1.0);
                draw_text(
                    text,
                    pref_size_f32.x / 2.0 - text_dimensions.width / 2.0,
                    pref_size_f32.y / 2.0,
                    16.0,
                    RED,
                );
            }
        }

        set_default_camera();
        clear_background(BLACK);
        crt_material.set_uniform("iTime", get_time() as f32);
        crt_material.set_uniform("iResolution", (pref_size_f32.x, pref_size_f32.y));
        gl_use_material(&crt_material);

        let screen_pad_x = (screen_width() - ((pref_size.x as f32) * (texel_size as f32))) * 0.5;
        let screen_pad_y = (screen_height() - ((pref_size.y as f32) * (texel_size as f32))) * 0.5;
        let dest_size = pref_size_f32 * vec2(texel_size as f32, texel_size as f32);

        draw_texture_ex(
            &main_render_target.texture,
            screen_pad_x,
            screen_pad_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(dest_size),
                ..Default::default()
            },
        );
        gl_use_default_material();

        draw_text(
            get_fps().to_string().as_str(),
            16.0,
            32.0,
            16.0,
            GREEN,
        );
        next_frame().await
    }
}
