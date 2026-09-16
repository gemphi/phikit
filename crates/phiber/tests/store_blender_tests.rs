use phiber::store::hosts::BHostType;
use phiber::store::panel_role::BPanelRole;
use phiber::store::parts::BPartType;
use phiber::store::regions::{BRegionAlignment, BRegionType};
use phiber::store::spaces::{BObjectMode, BSpaceType};
use phiber::store::spec::{BSpec, BSpecType};

#[test]
fn test_blender_52_region_types_and_alignments() {
    // Verify all 17 regions are present
    assert_eq!(BRegionType::all().len(), 17);

    // Verify DNA type IDs matching Blender 5.2 DNA_screen_types.h (eRegion_Type)
    assert_eq!(BRegionType::Window.dna_type_id(), 0);
    assert_eq!(BRegionType::Center.dna_type_id(), 0);
    assert_eq!(BRegionType::Header.dna_type_id(), 1);
    assert_eq!(BRegionType::Channels.dna_type_id(), 2);
    assert_eq!(BRegionType::Temporary.dna_type_id(), 3);
    assert_eq!(BRegionType::UI.dna_type_id(), 4);
    assert_eq!(BRegionType::Tools.dna_type_id(), 5);
    assert_eq!(BRegionType::ToolProps.dna_type_id(), 6);
    assert_eq!(BRegionType::Preview.dna_type_id(), 7);
    assert_eq!(BRegionType::HUD.dna_type_id(), 8);
    assert_eq!(BRegionType::NavBar.dna_type_id(), 9);
    assert_eq!(BRegionType::Execute.dna_type_id(), 10);
    assert_eq!(BRegionType::Footer.dna_type_id(), 11);
    assert_eq!(BRegionType::ToolHeader.dna_type_id(), 12);
    assert_eq!(BRegionType::XR.dna_type_id(), 13);
    assert_eq!(BRegionType::AssetShelf.dna_type_id(), 14);
    assert_eq!(BRegionType::AssetShelfHeader.dna_type_id(), 15);
    assert_eq!(BRegionType::Scrubbing.dna_type_id(), 16);

    // Verify default alignments matching Blender 5.2
    assert_eq!(
        BRegionType::Header.default_alignment(),
        BRegionAlignment::Top
    );
    assert_eq!(
        BRegionType::ToolHeader.default_alignment(),
        BRegionAlignment::Top
    );
    assert_eq!(
        BRegionType::Footer.default_alignment(),
        BRegionAlignment::Bottom
    );
    assert_eq!(
        BRegionType::Tools.default_alignment(),
        BRegionAlignment::Left
    );
    assert_eq!(BRegionType::UI.default_alignment(), BRegionAlignment::Right);
    assert_eq!(
        BRegionType::HUD.default_alignment(),
        BRegionAlignment::Float
    );
}

#[test]
fn test_blender_52_host_dna_space_types() {
    assert_eq!(BHostType::Empty.dna_space_type(), 0);
    assert_eq!(BHostType::View3D.dna_space_type(), 1);
    assert_eq!(BHostType::Ipo.dna_space_type(), 2);
    assert_eq!(BHostType::Outliner.dna_space_type(), 3);
    assert_eq!(BHostType::Buts.dna_space_type(), 4);
    assert_eq!(BHostType::File.dna_space_type(), 5);
    assert_eq!(BHostType::Image.dna_space_type(), 6);
    assert_eq!(BHostType::Info.dna_space_type(), 7);
    assert_eq!(BHostType::Seq.dna_space_type(), 8);
    assert_eq!(BHostType::Text.dna_space_type(), 9);
    assert_eq!(BHostType::Action.dna_space_type(), 12);
    assert_eq!(BHostType::Nla.dna_space_type(), 13);
    assert_eq!(BHostType::Node.dna_space_type(), 16);
    assert_eq!(BHostType::Logic.dna_space_type(), 17);
    assert_eq!(BHostType::Console.dna_space_type(), 18);
    assert_eq!(BHostType::Prefs.dna_space_type(), 19);
    assert_eq!(BHostType::Clip.dna_space_type(), 20);
    assert_eq!(BHostType::Topbar.dna_space_type(), 21);
    assert_eq!(BHostType::Statusbar.dna_space_type(), 22);
    assert_eq!(BHostType::Spreadsheet.dna_space_type(), 23);
}

#[test]
fn test_blender_52_workspace_modes() {
    assert_eq!(
        BSpaceType::Layout.default_object_mode(),
        BObjectMode::Object
    );
    assert_eq!(
        BSpaceType::Modeling.default_object_mode(),
        BObjectMode::Edit
    );
    assert_eq!(
        BSpaceType::Sculpting.default_object_mode(),
        BObjectMode::Sculpt
    );
    assert_eq!(
        BSpaceType::TexturePaint.default_object_mode(),
        BObjectMode::TexturePaint
    );
}

#[test]
fn test_bspec_hierarchy_and_roles() {
    let region_spec = BRegionType::Header.items();
    assert_eq!(region_spec.idx(), BSpecType::BRegion(BRegionType::Header));

    let host_spec = BHostType::View3D.items();
    assert_eq!(host_spec.idx(), BSpecType::BHost(BHostType::View3D));

    assert_eq!(BRegionType::UI.panel_role(), BPanelRole::Sidebar);
    assert_eq!(BRegionType::Tools.panel_role(), BPanelRole::Toolbar);
    assert_eq!(BPartType::AdjustLastOp.panel_role(), BPanelRole::ToolProps);
}
