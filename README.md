# Duden-rs

Get a Wort, its Bedeutungen and Beispiele from [Duden.de](https://www.duden.de/).

You can also get the Wort, Bedeutungen and Beispiele for the current [Wort-des-Tages](https://www.duden.de/wort-des-tages).

This crate provides a binary for displaying the Wort-des-Tages as well as for quering words, as well as provide a library for querying such words via [rust](https://www.rust-lang.org/).

## Using the binary

Having installed rust (for example via [rustup](https://rustup.rs/)), clone the repository and simply use `cargo run` in the repository.

Note that like in the library, for quering a Wort, you need to use its name as it would appear in the URL. For example,
while you would write "sit venia verbo", the url is `https://www.duden.de/rechtschreibung/sit_venia_verbo`, so you need to query `sit_venia_verbo`.

## Examples for the library

### Query the Wort "Apfel"

```rust
use duden_rs_lib::get_wort;

#[tokio::main]
async fn main() -> Result<(), String> {
    let wort = get_wort("Apfel").await?;

    println!("{}",wort);

    Ok(())
}
```

### Query the Wort-des-Tages

```rust
use duden_rs_lib::get_wort_des_tages;

#[tokio::main]
async fn main() -> Result<(), String> {
    let wort = get_wort_des_tages().await?;

    println!("{}",wort);

    Ok(())
}
```

### Accessing bedeutungen and beispiele

```rust
use duden_rs_lib::get_wort;

#[tokio::main]
async fn main() -> Result<(), String> {
    let wort = get_wort("Apfel").await?;

    // Correct spelling with correct accents and hyphenation.
    let wort_spelling = wort.wort;

    // A Wort has a vector of bedeutungen,
    // each with its own Bedeutungs string and vector of Beispiele.
    let erste_bedeutung = wort.bedeutungen.first().unwrap();

    // A Beispiel is always associated to a Bedeutung,
    // therefore it resides in a vector in the corresponding struct.
    let erstes_beispiel = some_bedeutung.beispiele.first().unwrap();

    println!("Für Wort {}", wort_spelling);
    println!("Gibt es die Beschreibung \"{}\"", erste_bedeutung.bedeutung);
    println!("Mit Beispiel \"{}\"",erstes_beispiel);

    Ok(())
}
```
