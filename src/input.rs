use bevy::prelude::*;
use std::process::exit;
use crate::Layout;

pub fn input_system(keyboard: Res<Input<KeyCode>>, mut layout_events: EventWriter<Layout>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit(0);
    }
    if keyboard.just_pressed(KeyCode::Key1) {
        layout_events.send(Layout::qwerty());
    }
    if keyboard.just_pressed(KeyCode::Key2) {
        layout_events.send(Layout::dvorak());
    }
    if keyboard.just_pressed(KeyCode::Key3) {
        layout_events.send(Layout::colemak());
    }
    if keyboard.just_pressed(KeyCode::Key4) {
        layout_events.send(Layout::colemak_dh());
    }
}
