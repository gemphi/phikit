//! Default state initialization set.

use super::key::BKey;
use crate::bui::theme::BTheme;
use crate::store::hosts::BHostType;
use crate::store::spaces::BSpaceType;
use crate::store::zones::BZoneType;
use std::collections::HashMap;

pub fn default_state() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert(BKey::CtxSpace.key(), format!("{:?}", BSpaceType::Layout));
    m.insert(BKey::CtxZone.key(), format!("{:?}", BZoneType::Center));
    m.insert(BKey::CtxHost.key(), format!("{:?}", BHostType::View3D));
    m.insert(BKey::CtxTool.key(), "Select Box".to_string());
    m.insert(BKey::CtxObject.key(), "Cube".to_string());
    m.insert(BKey::CtxScene.key(), "Scene".to_string());
    m.insert(BKey::CtxViewLayer.key(), "ViewLayer".to_string());
    m.insert(BKey::CtxFrame.key(), "1".to_string());
    m.insert(BKey::CtxPlaying.key(), "false".to_string());
    m.insert(BKey::Theme.key(), BTheme::default().name().to_string());
    m.insert(BKey::UseGradients.key(), "true".to_string());
    m
}
