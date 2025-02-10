use core::fmt;
use std::iter;

use select::{
    document::Document,
    node::Node,
    predicate::{Any, Attr, Child, Class, Descendant, Name, Not, Predicate},
};

/// A single Wort together with a list of Bedeutungen and matching Beispiele.
pub struct Wort {
    pub wort: String,
    pub bedeutungen: Vec<Bedeutung>,
}

/// A single Bedeutung for a Wort. May have a list of Beispiele.
pub struct Bedeutung {
    pub bedeutung: String,
    pub beispiele: Vec<String>,
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
            writeln!(f)?;
        }
        Ok(())
    }
}

/// Get the Duden Wort-des-Tages together with its Bedeutungen and Beispiele from https://www.duden.de/wort-des-tages
pub async fn get_wort_des_tages() -> Result<Wort, String> {
    let wort = get_wort_des_tages_link_name().await?;

    get_wort(&wort).await
}

/// Get a Wort together with its Bedeutungen and Beispiele from Duden.de.
/// The argument `wort` should be the name as it would be written in the Duden URL.
pub async fn get_wort(wort: &str) -> Result<Wort, String> {
    let doc = fetch_document_for_wort(wort).await?;

    let wort = get_wort_from_document(&doc)?;

    let bedeutungs_node = get_bedeutungs_node_from_document(&doc)?;

    let bedeutungen = get_bedeutungen_from_node(&bedeutungs_node)?;

    let beispiele = get_beispiele_from_node(&bedeutungs_node)?;

    Ok(Wort {
        wort,
        bedeutungen: bedeutungen
            .into_iter()
            .zip(beispiele.into_iter().chain(iter::repeat(vec![])))
            .map(|(bedeutung, beispiele)| Bedeutung {
                bedeutung,
                beispiele,
            })
            .collect::<Vec<_>>(),
    })
}

/// Get the Wort as it would appear in the link to its page in the Duden URL.
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

/// Get the corresponding document for a given Wort using the reqwest crate.
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

/// The correct spelling of a word can be found in a `meta` tag with `property=og:title`.
/// The argument `doc` should be the whole site document for a specific word from duden.de/rechtschreibung/{wort}.
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

    let mut wort = match wort_node.attr("content") {
        Some(value) => value,
        None => return Err("Scraping error while searching for Wort attribute on meta tag: meta tag does not have attribute \"content\"".to_string()),
    };

    wort = wort.split(" |").next().unwrap_or(wort);

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

/// Get a list of bedeutungen for the given Wort.
/// The argument `node` should be the bedeutungs node given by the function `get_bedeutungs_node_from_document`.
fn get_bedeutungen_from_node(node: &Node) -> Result<Vec<String>, String> {
    // We ignore every other element that is not related to a bedeutung
    // The HTML-tree can be very complicated but every non-related element is a descendant of these elements,
    // so we can filter for those
    let non_predicate = Not(Name("figcaption")
        .or(Name("header"))
        .or(Name("dl"))
        .or(Class("special-note"))
        .descendant(Any));

    let bedeutung_text_fragments = node.find(non_predicate);

    // now we can extract the remaining text of the relevant nodes
    let bedeutungs_text = bedeutung_text_fragments
        .map(|n| n.as_text().unwrap_or(""))
        .collect::<Vec<&str>>()
        .join(" ");

    // finally, every bedeutung is a continuos slice of text without linebreaks,
    // so we can split at newline characters to get the bedeutungen.
    // since there is a lot of empty text in between like whitespace and linebreaks,
    // we filter those irrelevant elements.
    let bedeutungen = bedeutungs_text
        .split('\n')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    Ok(bedeutungen)
}
