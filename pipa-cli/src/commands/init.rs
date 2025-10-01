use pipa::init::init_project;

match init_project() {
    Ok(msg) => println!("{}", msg),
    Err(e) => eprintln!("❌ {}", e),
}
