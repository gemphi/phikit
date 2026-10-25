//! `play.start` — the Play host's button: the picked match built off the
//! UI thread (`scene::play::arena`, on the figure in the embedded demo) and
//! opened as the scene, an untitled document at its first frame, playing.
//! `wear` — the Project Browser's footwear: a set's stage built the same
//! way (`scene::wear::stage`) and opened.

use super::super::BPhiberApp;
use crate::v2::scene::play;
use crate::v2::state::{CtxKey, log};
use gpui::{AppContext, Context};

impl BPhiberApp {
    pub(crate) fn play_match(&mut self, cx: &mut Context<Self>) {
        let choice = crate::v2::part::play::choice(&self.state);
        let summary = crate::v2::part::play::summary(&choice);
        log::push(format!("Building the match: {summary}"));
        cx.spawn(async move |this, cx| {
            let started = web_time::Instant::now();
            let built = cx.background_spawn(async move { play::demo().and_then(|demo| play::arena(&choice, play::SECONDS, &demo)) }).await;
            this.update(cx, |this, cx| {
                match built {
                    Some(mut model) => {
                        model.report.duration_ms = started.elapsed().as_millis() as u64;
                        log::push(format!("Match ready ({} ms): {} objects, {} players", model.report.duration_ms, model.objects.len(), model.motion.rigs.rigs.len()));
                        let outcome = super::super::import::ImportOutcome::Loaded(model);
                        if let Some(model) = super::super::import::settle_import(&mut this.state, outcome) {
                            this.adopt_scene(format!("play:{summary}"), std::sync::Arc::new(model));
                            this.untitled = false;
                            this.state.set(CtxKey::Frame, "1".to_string());
                            this.state.set(CtxKey::Playing, "true".to_string());
                        }
                    }
                    None => log::push("The match could not be built: the demo's figure is missing".to_string()),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub(crate) fn open_wear(&mut self, set: String, cx: &mut Context<Self>) {
        let named = phiplay::kit::footwear::BFootwearSet::by_slug(&set);
        let title = named.map_or("Footwear", |s| s.spec().label);
        log::push(format!("Building the footwear stage: {title}"));
        cx.spawn(async move |this, cx| {
            let built = cx.background_spawn(async move { crate::v2::scene::wear::open(&set) }).await;
            this.update(cx, |this, cx| {
                match built {
                    Ok(model) => {
                        log::push(format!("Footwear ready: {} objects", model.objects.len()));
                        let outcome = super::super::import::ImportOutcome::Loaded(model);
                        if let Some(model) = super::super::import::settle_import(&mut this.state, outcome) {
                            this.adopt_scene(format!("wear:{title}"), std::sync::Arc::new(model));
                            this.untitled = false;
                            this.state.set(CtxKey::Frame, "1".to_string());
                        }
                    }
                    Err(why) => log::push(format!("The footwear could not be built: {why}")),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}
