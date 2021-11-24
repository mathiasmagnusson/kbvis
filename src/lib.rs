use bevy::prelude::*;
use bevy::text::Text2dSize;

mod input;
mod data;

pub fn run() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_event::<Layout>()
        .add_startup_stage("keys", SystemStage::single(spawn_keys_system.system()))
        .add_system(input::input_system.system())
        .add_system(spawn_bigram_arrows.system())
        .add_system(assign_key_texts_system.system())
        .add_system(assign_key_colors_system.system())
        .add_system(arrow_movement_system.system())
        .run();
}

pub struct Arrow(Key, Key, f32);

pub struct Layout(&'static str);

impl Layout {
    pub fn qwerty() -> Self {
        Self("QWERTYUIOPASDFGHJKL;ZXCVBNM,./")
    }
    pub fn dvorak() -> Self {
        Self("',.PYFGCRLAOEUIDHTNS;QJKXBMWVZ")
    }
    pub fn colemak() -> Self {
        Self("QWFPGJLUY;ARSTDHNEIOZXCVBKM,./")
    }
    pub fn colemak_dh() -> Self {
        Self("QWFPBJLUY;ARSTGMNEIOZXCDVKH,./")
    }
    pub fn s(&self, i: usize) -> &'static str {
        &self.0[i..i+1]
    }
    pub fn ch(&self, i: usize) -> char {
        self.0.chars().nth(i).unwrap()
    }
    pub fn find(&self, c: char) -> Key {
        Key(self.0.find(c).unwrap())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Key(usize);

const KEY_SIZE: f32 = 100.0;
const PADDING: f32 = 10.0;

impl Key {
    pub fn pos(&self) -> Vec3 {
        let half_width = (KEY_SIZE * 10.0 + PADDING * 9.0) / 2.0;
        let half_height = (KEY_SIZE * 3.0 + PADDING * 2.0) / 2.0;
        Vec3::new(
            (self.0 % 10) as f32 * (KEY_SIZE + PADDING) - half_width + KEY_SIZE / 2.0,
            half_height - KEY_SIZE / 2.0 - (self.0 / 10) as f32 * (KEY_SIZE + PADDING),
            0.0,
        )
    }
}

fn spawn_bigram_arrows(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut layout_events: EventReader<Layout>,
    arrows: Query<Entity, With<Arrow>>,
) {
    let layout = match layout_events.iter().last() {
        Some(layout) => layout,
        None => return,
    };
    for entity in arrows.iter() {
        commands.entity(entity).despawn_recursive();
    }
    let distrib = data::get_2_gram_distrib();
    for ((fst, snd), d) in distrib {
        if d < 0.2 || fst == snd { continue; }
        let a = Color::GREEN;
        let b = Color::GREEN * 0.25 + Color::BLUE * 0.75;
        let color = a * d + b * (1.0 - d);
        spawn_arrow(&mut commands, materials.add(color.into()), layout.find(fst), layout.find(snd), d * 20.0);
    }
}

fn assign_key_colors_system(
    text_query: Query<(&Parent, &Key)>,
    mut key_query: Query<&mut Handle<ColorMaterial>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut layout_events: EventReader<Layout>,
) {
    let layout = match layout_events.iter().last() {
        Some(layout) => layout,
        None => return,
    };
    let distrib = data::get_1_gram_distrib();
    for (parent, &Key(key)) in text_query.iter() {
        let material: &mut Handle<ColorMaterial> = &mut key_query.get_mut(parent.0).unwrap();

        let color = distrib.get(&layout.ch(key)).map_or(Color::GRAY, |&d|
            Color::WHITE * (1.0 - d) + Color::RED * d
        );
        *material = materials.add(color.into());
    }
}

fn assign_key_texts_system(
    mut query: Query<(&mut Text, &Key)>,
    mut layout_events: EventReader<Layout>,
) {
    let layout = match layout_events.iter().last() {
        Some(layout) => layout,
        None => return,
    };
    for (mut text, &Key(key)) in query.iter_mut() {
        text.sections[0].value = layout.s(key).to_string();
    }
}

fn spawn_keys_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("/usr/share/fonts/truetype/liberation2/LiberationMono-Regular.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 70.0,
        color: Color::BLACK,
    };
    let text_align = TextAlignment {
        vertical: VerticalAlign::Center,
        horizontal: HorizontalAlign::Center,
    };
    for key in (0..30).map(Key) {
        commands.spawn()
            .insert_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(KEY_SIZE, KEY_SIZE)),
                material: materials.add(Color::GRAY.into()),
                transform: Transform::from_translation(key.pos()),
                ..Default::default()
            }).with_children(|parent| {
                parent.spawn()
                    .insert(key)
                    .insert_bundle(Text2dBundle {
                        text: Text::with_section(".", text_style.clone(), text_align),
                        text_2d_size: Text2dSize { size: Size::new(KEY_SIZE, KEY_SIZE) },
                        transform: Transform::from_xyz(0.0, 0.0, 2.0),
                        ..Default::default()
                    });
            });
    }

    commands.spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d());
}

fn spawn_arrow(commands: &mut Commands, material: Handle<ColorMaterial>, from: Key, to: Key, thiccness: f32) {
    commands.spawn()
        .insert_bundle(SpriteBundle {
            material: material.clone(),
            ..Default::default()
        })
        .insert(Arrow(from, to, thiccness))
        .with_children(|parent| {
            parent.spawn().insert_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(thiccness * 3.0, thiccness)),
                material: material.clone(),
                ..Default::default()
            });
        });
}

fn arrow_movement_system(
    mut arrows: Query<(&mut Transform, &mut Sprite, &Children, &Arrow)>,
    mut tips: Query<&mut Transform, Without<Arrow>>,
    time: Res<Time>,
) {
    for (mut transform, mut sprite, children, arrow) in arrows.iter_mut() {
        let r = |seed| {
            let p = perlin_rust::PerlinNoise::new(seed + arrow.0.pos().x as f64 + arrow.0.pos().y as f64 + arrow.1.pos().x as f64 + arrow.1.pos().y as f64);
            (p.perlin2(time.time_since_startup().as_secs_f64() * 0.5, 0.0) * 60.0) as f32
        };
        let from = arrow.0.pos() + Vec3::new(r(456.0), r(41564.0), 0.0);
        let to = arrow.1.pos() + Vec3::new(r(1.16), r(-20.0), 0.0);

        let v = to - from;
        let dist = v.length();
        let a = v.y.atan2(v.x);

        let mid = (from + to) / 2.0;

        sprite.size = Vec2::new(arrow.2, dist);
        transform.translation = Vec3::new(mid.x, mid.y, 1.0);
        transform.rotation = Quat::from_rotation_z(a - std::f32::consts::FRAC_PI_2);

        let mut tip_transform = match tips.get_mut(children[0]) {
            Ok(tip) => tip,
            Err(_) => continue,
        };

        tip_transform.translation = Vec3::new(0.0, (dist - arrow.2) / 2.0, 0.0);
    }
}
