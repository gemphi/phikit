fn view(role: Role, c: &Ctx) -> Element {
    match role {
        Role::Row => row::BRow.render(c),
        Role::Card => card::BCard { role }.render(c),
    }
}
