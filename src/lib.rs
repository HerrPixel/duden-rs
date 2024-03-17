use select::{
    document::Document,
    predicate::{Attr, Class, Name, Predicate},
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

pub async fn get_wort_des_tages() -> Result<Wort, String> {
    let wort = get_wort_des_tages_link_name().await?;

    get_wort(&wort).await
}

pub async fn get_wort(wort: &str) -> Result<Wort, String> {
    let doc = fetch_document_for_wort(wort).await?;

    Err("lol".to_string())
}

async fn get_wort_des_tages_link_name() -> Result<String, String> {
    let url = "https://www.duden.de/wort-des-tages";

    let fetched_html_response = match reqwest::get(url).await {
        Ok(response) => response,
        Err(e) => {
            return Err(format!(
                "Reqwest error while fetching Wort-des-Tages site: {}",
                e
            ))
        }
    };

    let fetched_html = match fetched_html_response.text().await {
        Ok(text) => text,
        Err(e) => {
            return Err(format!(
                "Reqwest error while fetching text from Wort-des-Tages site: {}",
                e
            ))
        }
    };

    let document = Document::from(fetched_html.as_str());

    // the link for the description page of the Wort-des-Tages is contained in this element
    let link_predicate = Name("a").and(Class("scene__title-link"));

    // find the node first
    let link_node = match document.find(link_predicate).next() {
        Some(node) => node,
        None => return Err(
            "Scraping error while searching for Wort-des-Tages Wort-Link: \"a\" element not found"
                .to_string(),
        ),
    };

    // then extract the link
    let link = match link_node.attr("href") {
        Some(value) => value,
        None => return Err(
            "Scraping error while searching for Wort-des-Tages Wort-Link: \"a\" element does not have \"href\" attribute"
                .to_string(),
        ),
    };

    // and finally strip the prefix to be left with the word as it will be written in the url
    let link_name = match link.strip_prefix("/rechtschreibung/") {
        Some(s) => s,
        None => return Err(
            "Scraping error while searching for Wort-des-Tages Wort-Link: link does not have correct scheme \"/rechtschreibung/{wort}\""
                .to_string(),
        ),
    };

    Ok(link_name.to_string())
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
