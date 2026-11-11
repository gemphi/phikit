use super::*;

#[test]
fn paths_split_into_fields_keys_and_indices() {
    let p = BRnaPath::parse(r#"pose.bones["a \"b\""].constraints["Armature"].targets[3].weight"#).unwrap();
    assert_eq!(
        p.0,
        vec![
            BRnaStep::Field("pose".into()),
            BRnaStep::Field("bones".into()),
            BRnaStep::Key("a \"b\"".into()),
            BRnaStep::Field("constraints".into()),
            BRnaStep::Key("Armature".into()),
            BRnaStep::Field("targets".into()),
            BRnaStep::Index(3),
            BRnaStep::Field("weight".into()),
        ]
    );
    assert_eq!(BRnaPath::parse(r#"pose.bones["x"]["IK_FK"]"#).unwrap().0.last(), Some(&BRnaStep::Key("IK_FK".into())));
}
