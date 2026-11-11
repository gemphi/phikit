use super::*;

fn run(text: &str, vars: &[(&str, f64)]) -> Option<f64> {
    let names: Vec<String> = std::iter::once("frame".to_string()).chain(vars.iter().map(|v| v.0.to_string())).collect();
    let values: Vec<f64> = std::iter::once(1.0).chain(vars.iter().map(|v| v.1)).collect();
    BExpr::parse(text, &names)?.eval(&values)
}

#[test]
fn rigify_expressions_evaluate_as_python_would() {
    assert_eq!(run("var == 1", &[("var", 1.0)]), Some(1.0));
    assert_eq!(run("var == 2", &[("var", 1.0)]), Some(0.0));
    let v = run("(1-sy)*pi", &[("sy", 0.25)]).unwrap();
    assert!((v - 0.75 * std::f64::consts::PI).abs() < 1e-12);
    assert_eq!(run("1 < var < 3", &[("var", 2.0)]), Some(1.0));
    assert_eq!(run("1 < var < 3", &[("var", 3.0)]), Some(0.0));
    assert_eq!(run("0 or var and 5", &[("var", 2.0)]), Some(5.0));
    assert_eq!(run("2 if var > 1 else -3", &[("var", 0.0)]), Some(-3.0));
    assert_eq!(run("max(1, var, 0.5) + clamp(var, 0, 1.5)", &[("var", 2.0)]), Some(3.5));
    assert_eq!(run("1 / var", &[("var", 0.0)]), None, "division by zero flags the driver");
    assert!(BExpr::parse("var ** 2", &["var".into()]).is_none(), "not a simple expression");
    assert!(BExpr::parse("bpy.data", &[]).is_none());
}
