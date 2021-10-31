use game::Game;
use std::mem;

fn main() {
    println!("Size of Game<3>: {} bytes", mem::size_of::<Game<3>>(),);
    println!("Size of Game<4>: {} bytes", mem::size_of::<Game<4>>(),);
    println!("Size of Game<5>: {} bytes", mem::size_of::<Game<5>>(),);
    println!("Size of Game<6>: {} bytes", mem::size_of::<Game<6>>(),);
}
