use core::fmt;

use select::{
    document::Document,
    node::Node,
    predicate::{Any, Attr, Child, Class, Descendant, Name, Not, Predicate},
};

pub struct Wort {
    wort: String,
    bedeutungen: Vec<Bedeutung>,
}
pub struct Bedeutung {
    bedeutung: String,
    beispiele: Vec<String>,
}

impl fmt::Display for Wort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Wort: {}", self.wort)?;
        writeln!(f, "Bedeutung(en):")?;
        for bedeutung in &self.bedeutungen {
            writeln!(f, "    {}", bedeutung.bedeutung)?;
            writeln!(f, "    Beispiel(e):")?;
            for beispiel in &bedeutung.beispiele {
                writeln!(f, "        {}", beispiel)?;
            }
        }
        Ok(())
    }
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

/// The bedeutungen and beispiele are always contained in a `div` element with `id=bedeutung` or `div=bedeutungen`.
/// This function returns that node.
fn get_bedeutungs_node_from_document(doc: &Document) -> Result<Node, String> {
    let bedeutungs_node_predicate = Name("div").and(Attr("id", "bedeutung"));

    let mut bedeutungs_node = doc.find(bedeutungs_node_predicate).next();

    // depending on the number of "beispiele", the id could be singular or plural,
    // therefore if the first did not find a corresponding node, we try the other version
    if bedeutungs_node.is_none() {
        let bedeutungs_node_predicate = Name("div").and(Attr("id", "bedeutungen"));
        bedeutungs_node = doc.find(bedeutungs_node_predicate).next();

        if bedeutungs_node.is_none() {
            return Err("Scraping error while searching for bedeutungs node: \"div\" element with id \"bedeutung(en)\" not found".to_string());
        }
    }

    Ok(bedeutungs_node.unwrap())
}

/// Get a list of list of beispiele for the given Wort.
/// Each list of beispiele corresponds to one bedeutung given by `get_bedeutungen_from_node`.
/// The argument `node` should be the bedeutungs node given by the function `get_bedeutungs_node_from_document`.
fn get_beispiele_from_node(node: &Node) -> Result<Vec<Vec<String>>, String> {
    // Every beispiel list is a definition list.
    let beispiel_list_selector = Name("dl");

    // But not every definition list is a beispiel list, only if the title of that list is "Beispiel(e)".
    // Therefore we extract the title first (given by a definition term element),
    // and test if it is either "Beispiel" or "Beispiele".
    let list_title_selector = Child(Name("dl"), Name("dt"));

    // Finally, every beispiel is then found in a list item element
    let beispiel_selector = Descendant(Name("dl"), Name("li"));

    let mut extracted_beispiele = Vec::new();

    for beispiel_list in node.find(beispiel_list_selector) {
        let list_title = beispiel_list.find(list_title_selector).next();

        if list_title.is_none() {
            continue;
        }

        // the list could be a "Sprichwörter" or "Redewendungen" list, we are not interested in those.
        if list_title.unwrap().text() != "Beispiel" && list_title.unwrap().text() != "Beispiele" {
            continue;
        }

        let mut beispiele_of_current_list = Vec::new();

        // extracting the text from list item descendants yields the beispiele.
        for beispiel in beispiel_list.find(beispiel_selector) {
            beispiele_of_current_list.push(beispiel.text());
        }

        extracted_beispiele.push(beispiele_of_current_list);
    }

    Ok(extracted_beispiele)
}
