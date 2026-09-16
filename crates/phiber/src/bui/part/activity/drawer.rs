//! Secondary slide-out drawer for social chat, workspace switcher, or AI stream.

use super::super::super::bview::{BAction, BView, CardShade};
use super::super::super::state::{BKey, BState};
use crate::store::spaces::BSpaceType;

pub struct ActivityDrawer;

impl ActivityDrawer {
    pub fn build(state: &BState, active_tab: &str) -> BView {
        match active_tab {
            "chat" => {
                let active_chat = state.get(&BKey::ActiveChatUser).unwrap_or("@alice");
                BView::card()
                    .with_shade(CardShade::Activity)
                    .with_width(Some(220.0))
                    .with_header("DIRECT MESSAGES")
                    .with_badge("3 Online")
                    .with_tool(BView::button().with_icon("plus"))
                    .with_body(
                        BView::list()
                            .with_gap(Some(2.0))
                            .with_item(
                                BView::item("@alice (3D Sculptor)")
                                    .with_icon("user")
                                    .with_badge("♀ Online")
                                    .with_active(active_chat == "@alice")
                                    .with_action(BAction::Set(
                                        BKey::ActiveChatUser,
                                        "@alice".to_string(),
                                    )),
                            )
                            .with_item(
                                BView::item("@bob (Shader Artist)")
                                    .with_icon("user")
                                    .with_badge("♂ Online")
                                    .with_active(active_chat == "@bob")
                                    .with_action(BAction::Set(
                                        BKey::ActiveChatUser,
                                        "@bob".to_string(),
                                    )),
                            )
                            .with_item(
                                BView::item("@carol (Animator)")
                                    .with_icon("user")
                                    .with_badge("♀ Away")
                                    .with_action(BAction::Set(
                                        BKey::ActiveChatUser,
                                        "@carol".to_string(),
                                    )),
                            ),
                    )
                    .with_footer("Connected to #viewport-collab")
            }
            _ => {
                let active_space = state.get(&BKey::CtxSpace).unwrap_or("Layout");
                let space_items: Vec<BView> = BSpaceType::all()
                    .iter()
                    .map(|&st| {
                        let name = format!("{:?}", st);
                        let is_active = name == active_space;
                        BView::item(st.label())
                            .with_icon(st.icon())
                            .with_active(is_active)
                            .with_action(BAction::Set(BKey::CtxSpace, name))
                    })
                    .collect();

                BView::card()
                    .with_shade(CardShade::Activity)
                    .with_width(Some(220.0))
                    .with_header("WORKSPACES")
                    .with_body(BView::list().with_gap(Some(2.0)).with_items(space_items))
                    .with_footer(format!("Active: {}", active_space))
            }
        }
    }
}
