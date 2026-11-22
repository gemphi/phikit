//! A shop, as its own file has it (`puikit/app/phiogy/public/data/shop`):
//! its products once, in a list; its menu, departments as headings and
//! their categories as pages, each naming its products by their place
//! in that list; and what its landing shows. It is read when the shop
//! is opened, by the names of `data/site.tsv`.

use appkit::service::PTable;
use serde_json::Value;

use super::Face;

/// A thing of a shop.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Product {
    pub id: String,
    pub title: String,
    /// Its price as the shop writes it, and as a number of dollars.
    pub sub: String,
    pub cost: f64,
    pub tag: String,
    pub kind: String,
    pub meta: String,
    /// Its picture by its key, or a cell of a sheet.
    pub image: String,
    pub sprite: Option<Face>,
    /// Whether Phiora knows its price, and whether it can be worn.
    pub look: bool,
    pub wear: bool,
    /// The few colours a theme is made of, as the web writes colours.
    pub swatch: Vec<String>,
    /// On its own page: its views, each with what it is called; what
    /// is said of it; and its facts, a name and a value.
    pub view: Vec<(String, String)>,
    pub body: String,
    pub field: Vec<(String, String)>,
}

/// A page of a shop's menu: its id, what it is called, its icon, the
/// heading it stands under, and the places of its products.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Shelf {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub head: String,
    pub at: Vec<usize>,
}

/// A shop.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Shop {
    pub id: String,
    pub label: String,
    pub sub: String,
    pub icon: String,
    pub shelf: Vec<Shelf>,
    pub product: Vec<Product>,
    /// Its landing: a picture, what the grid under it is called, and
    /// the places of that grid's products.
    pub image: String,
    pub lead: String,
    pub first: Vec<usize>,
}

impl Shop {
    /// A shop out of the text of its own file.
    pub fn read(text: &str, key: &PTable) -> Option<Shop> {
        let doc: Value = serde_json::from_str(text).ok()?;
        let word = |value: &Value, name: &str| value.get(key.get(name)).and_then(Value::as_str).unwrap_or_default().to_string();
        let list = |value: &Value, name: &str| value.get(key.get(name)).and_then(Value::as_array).cloned().unwrap_or_default();
        let flag = |value: &Value, name: &str| value.get(key.get(name)).is_some_and(|value| !value.is_null() && *value != Value::Bool(false));
        let place = |value: &Value| list(value, "shop.at").iter().filter_map(Value::as_u64).map(|at| at as usize).collect::<Vec<_>>();
        let pair = |one: &Value| Some((one.get(0)?.as_str()?.to_string(), one.get(1)?.as_str()?.to_string()));
        let product = |one: &Value| {
            let page = one.get(key.get("shop.page")).cloned().unwrap_or_default();
            let sprite = one.get(key.get("shop.sprite")).filter(|sprite| sprite.is_object());
            let cell = |sprite: &Value| Face { sheet: word(sprite, "sheet"), column: sprite.get(key.get("sheet.column")).and_then(Value::as_u64).unwrap_or_default() as u32, index: sprite.get(key.get("mark.index")).and_then(Value::as_u64).unwrap_or_default() as u32 };
            Product {
                id: word(one, "id"),
                title: word(one, "shop.title"),
                sub: word(one, "sub"),
                cost: one.get(key.get("shop.cost")).and_then(Value::as_f64).unwrap_or_default(),
                tag: word(one, "tag"),
                kind: word(one, "kind"),
                meta: word(one, "shop.meta"),
                image: word(one, "image"),
                sprite: sprite.map(cell),
                look: flag(one, "shop.look"),
                wear: flag(one, "shop.wear"),
                swatch: list(one, "shop.swatch").iter().filter_map(Value::as_str).map(str::to_string).collect(),
                view: list(&page, "shop.view").iter().map(|view| (word(view, "label"), word(view, "image"))).collect(),
                body: word(&page, "shop.body"),
                field: list(&page, "shop.field").iter().filter_map(pair).collect(),
            }
        };
        // A department is a heading over its categories; a page with no
        // department stands under none.
        let shelf = |head: &str, one: &Value| Shelf { id: word(one, "id"), label: word(one, "label"), icon: word(one, "icon"), head: head.to_string(), at: place(one) };
        let menu = list(&doc, "shop.menu");
        let shelves = menu.iter().flat_map(|one| match list(one, "item").is_empty() {
            true => vec![shelf("", one)],
            false => list(one, "item").iter().map(|under| shelf(&word(one, "label"), under)).collect(),
        });
        let landing = list(&doc, "shop.landing");
        let hero = landing.iter().find(|block| !word(block, "image").is_empty()).cloned().unwrap_or_default();
        let grid = landing.iter().find(|block| block.get(key.get("shop.at")).is_some()).cloned().unwrap_or_default();
        Some(Shop { id: word(&doc, "id"), label: word(&doc, "label"), sub: word(&doc, "sub"), icon: word(&doc, "icon"), shelf: shelves.collect(), product: list(&doc, "shop.product").iter().map(product).collect(), image: word(&hero, "image"), lead: word(&grid, "shop.title"), first: place(&grid) })
    }
}
