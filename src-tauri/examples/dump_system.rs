use gamefic_lib::system;

fn main() {
    let snapshot = system::snapshot();
    println!("{:#?}", snapshot);
}
