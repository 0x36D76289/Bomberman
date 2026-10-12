use crate::{
    app_state::{Ctx, POP_TO_MENU, Transition},
    graphics::text::FontId,
    ui::{
        UiState,
        draw::TextStyle,
        layout::Rect,
        style,
        textures::{Icon, UiTex},
        widgets::Action,
    },
};
use glam::Vec2;

pub fn tick(state: &mut UiState, ctx: &mut Ctx) -> Transition {
    let mut ui = super::begin(state, ctx);
    let screen = ui.screen();

    ui.backdrop();

    let mut y = screen.y * 0.55 - ui.time() * 75.0;
    let x = screen.x / 2.0;

    ui.dl.image_fit(
        UiTex::Icon(Icon::Trophy),
        Rect::centered(Vec2::new(x, y), Vec2::splat(140.0)),
        style::YELLOW,
    );
    y += 150.0;
    ui.dl.text(
        "BOMBERMAN",
        Vec2::new(x, y),
        TextStyle::new(FontId::Title, 110.0, style::YELLOW).outline(style::INK, 7.0),
    );
    y += 180.0;
    for (title, lines) in [
        (
            "Un projet 42 Paris",
            &["Écrit en Rust avec Vulkan"] as &[&str],
        ),
        ("Équipe", &["snaji", "cfrancie", "mduaka"]),
        (
            "Modèles 3D, sons et interface",
            &[
                "Kenney - www.kenney.nl",
                "Platformer Kit, UI Pack, Game Icons,",
                "Input Prompts, packs audio",
            ],
        ),
        (
            "Musique",
            &[
                "Juhani Junkala",
                "Chiptune Adventures",
                "Retro Game Music Pack",
            ],
        ),
        ("Textures", &["ambientCG - ambientcg.com"]),
        (
            "Polices",
            &["Lilita One par Juan Montoreano (OFL)", "Kenney Future"],
        ),
        (
            "Bibliothèques",
            &[
                "vulkano, winit, rodio, gilrs, glam",
                "tobj, image, fontdue, serde, rand",
            ],
        ),
    ] {
        ui.dl.text(
            title,
            Vec2::new(x, y),
            TextStyle::new(FontId::Title, 52.0, style::YELLOW).outline(style::INK, 4.0),
        );
        y += 80.0;
        for line in lines {
            ui.dl.text(
                line,
                Vec2::new(x, y),
                TextStyle::new(FontId::Title, 38.0, style::WHITE),
            );
            y += 54.0;
        }
        y += 90.0;
    }
    y += 200.0;
    let end = y.max(screen.y / 2.0);
    ui.dl.text(
        "Merci d'avoir joué !",
        Vec2::new(x, end - 45.0),
        TextStyle::new(FontId::Title, 72.0, style::WHITE).outline(style::INK, 5.0),
    );
    ui.dl.text(
        "(met outstanding stp)",
        Vec2::new(x, end + 45.0),
        TextStyle::new(FontId::Title, 72.0, style::WHITE).outline(style::INK, 5.0),
    );

    ui.dl.gradient(
        Rect::new(0.0, 0.0, screen.x, 160.0),
        style::BACKDROP_TOP,
        style::fade(style::BACKDROP_TOP, 0.0),
    );
    ui.dl.gradient(
        Rect::new(0.0, screen.y - 160.0, screen.x, 160.0),
        style::fade(style::BACKDROP_BOTTOM, 0.0),
        style::BACKDROP_BOTTOM,
    );
    ui.prompts(&[(Action::Back, "Retour")]);

    let back = ui.back_pressed() || ui.input.confirm || ui.input.mouse_released;
    ui.end(ctx.audio);
    if y < screen.y / 2.0 - 400.0 || back {
        (None, POP_TO_MENU)
    } else {
        (None, 0)
    }
}
