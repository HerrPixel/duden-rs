use duden_rs_lib::get_wort_des_tages;
use std::io;

use duden_rs_lib::get_wort;

#[tokio::main]
async fn main() {
    println!("Wort des Tages:");

    match get_wort_des_tages().await {
        Ok(s) => println!("{}", s),
        Err(e) => eprintln!("{}", e),
    };

    loop {
        println!("Please insert word as it would appear in the url:");

        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        match get_wort(&input).await {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("{}", e),
        };
    }
}
