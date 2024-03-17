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

pub async fn get_wort(wort: &str) -> Result<Wort, String> {
    let doc = fetch_document_for_wort(wort).await?;

    Err("lol".to_string())
}
async fn fetch_document_for_wort(wort: &str) -> Result<Document, String> {
    let url = format!("https://www.duden.de/rechtschreibung/{}", wort);

    let fetched_html_response = match reqwest::get(url).await {
        Ok(response) => response,
        Err(e) => return Err(format!("Reqwest error while fetching site: {}", e)),
    };

    let fetched_html = match fetched_html_response.text().await {
        Ok(text) => text,
        Err(e) => {
            return Err(format!(
                "Reqwest error while fetching text from site: {}",
                e
            ))
        }
    };

    Ok(Document::from(fetched_html.as_str()))
}
fn get_wort_from_document(doc: &Document) -> Result<String, String> {
    let meta_tag_predicate = Name("meta").and(Attr("property", "og:title"));

    let wort_node =
        match doc.find(meta_tag_predicate).next() {
            Some(node) => node,
            None => return Err(
                "Scraping error while searching for Wort meta tag: meta tag \"og:title\" not found"
                    .to_string(),
            ),
        };

    let wort = match wort_node.attr("content") {
        Some(value) => value,
        None => return Err("Scraping error while searching for Wort attribute on meta tag: meta tag does not have attribute \"content\"".to_string()),
    };

    Ok(wort.to_string())
}
