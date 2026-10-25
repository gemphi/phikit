//! What a channel row is.

use crate::phiphx::blend::BNlaTrack;

#[derive(Clone, Debug, PartialEq)]
pub enum BRowKind {
    /// `ANIMTYPE_SUMMARY`: every shown key. Collapsed in the Timeline,
    /// expanded — and so given its backdrop — in the Dope Sheet.
    Summary { expanded: bool },
    /// `ANIMTYPE_OBJECT`: one shown object. `expanded` is Blender's
    /// `ACHANNEL_SETTING_EXPAND` (`ADT_UI_EXPANDED`): closed, the object's
    /// keys are summarised onto its own row; open, its F-curves follow it as
    /// `Curve` rows and the object keeps the summary above them, which is
    /// what `ANIM_animdata_filter` emits for an expanded object.
    Object { name: String, expanded: bool },
    /// `ANIMTYPE_ACTION_SLOT`: in the Action Editor, one slot of the active
    /// object's action — named for what it animates, its ID type shown at
    /// the right of the channel — over its groups and curves. `object` is
    /// who it animates (a bound user, else the slot's own name), for its
    /// picks and its groups' folds; open until shut (`Slot::Flags::Expanded`
    /// on a new slot), its fold keyed by action and slot
    /// ([`BTimeRow::slot_key`]).
    Slot { object: String, action: String, identifier: String, expanded: bool },
    /// `ANIMTYPE_GROUP`: one `bActionGroup` of an expanded object — a bone's
    /// curves, or `Object Transforms` — summarising its curves' keys, and
    /// unfolding to them as `Curve` rows (`AGRP_EXPANDED`, closed until
    /// asked, keyed by object and group: [`BTimeRow::group_key`]).
    Group { object: String, name: String, expanded: bool },
    /// `ANIMTYPE_FCURVE`: one F-curve under an expanded object, named the way
    /// `getname_anim_fcurve` names it (`name::fcurve_name`) and carrying the
    /// curve's own colour for the swatch beside it. `grouped` when it sits
    /// under a `Group` row, one step further in.
    Curve { name: String, color: [f32; 3], grouped: bool },
    /// `ANIMTYPE_NLAACTION`: an object's action line in the NLA editor —
    /// the active action, or `<No Action>`; `tweaking` while
    /// `ADT_NLA_EDIT_ON`.
    Action { name: Option<String>, tweaking: bool },
    /// `ANIMTYPE_NLATRACK`: one NLA track with its strips.
    Track(BNlaTrack),
    /// `ANIMTYPE_FCURVE` in the Drivers editor: one driver, named as
    /// `getname_anim_fcurve` names it, with the curve's colour for the
    /// swatch beside the name.
    Driver { name: String, color: [f32; 3] },
}
