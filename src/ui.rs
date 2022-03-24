use bevy::prelude::*;
use log::{debug, error, info, trace, warn};

use crate::{Index, PlayerHitEvent};

pub struct PlayerHP(i16);

impl Default for PlayerHP {
    fn default() -> Self {
        PlayerHP(3)
    }
}

pub fn player_hit_handler(
    mut events: EventReader<PlayerHitEvent>,
    mut hp: Local<PlayerHP>,
    sheets: Res<Assets<TextureAtlas>>,
    mut q: Query<(&mut TextureAtlasSprite, &Handle<TextureAtlas>, &Index)>,
) {
    for _ev in events.iter() {
        hp.0 -= 1;
        info!("Player HP is {}", hp.0);

        if hp.0 == 0 {
            info!("Player died");
        } else if hp.0 < 0 {
            error!("This shouldn't happen in the real game");
            return;
        }

        for (mut heart_sprite, handle, i) in q.iter_mut() {
            if i.0 == hp.0 as i32 {
                let atlas = sheets.get(handle).unwrap();
                heart_sprite.index = (heart_sprite.index + 1) % atlas.textures.len();
            }
        }
    }
}
