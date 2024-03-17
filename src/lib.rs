use select::{
    document::Document,
    predicate::{Attr, Name, Predicate},
};

pub struct Wort {
    wort: String,
    bedeutungen: Vec<Bedeutung>,
}
pub struct Bedeutung {
    bedeutung: String,
    beispiele: Vec<String>,
}

/*

A little note about select:

While the library is good, the documentation is lacking a bit, therefore here are some ressources:
- How to use select for almost all use cases: https://proxiesapi.com/articles/the-ultimate-select-rs-cheat-sheet-for-rust
- Official docs just in case: https://docs.rs/select/latest/select/index.html

*/

fn get_wort(doc: &Document) -> Result<String, String> {
    let meta_tag_predicate = Name("meta").and(Attr("property", "og:title"));

    let wort_node = match doc.find(meta_tag_predicate).next() {
        Some(node) => node,
        None => return Err("Could not find Wort: meta tag \"og:title\" not found".to_string()),
    };

    let wort = match wort_node.attr("content") {
        Some(value) => value,
        None => {
            return Err(
                "Could not find Wort: meta tag does not have attribute \"content\"".to_string(),
            )
        }
    };

    Ok(wort.to_string())
}
