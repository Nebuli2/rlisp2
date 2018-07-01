extern crate im;
use im::ConsList;

mod context;
mod expression;

fn main() {
    let list: ConsList<i32> = ConsList::new();
    let list = list.cons(5).cons(2).cons(1);
    while let Some((head, list)) = list.uncons() {
        println!("{:?}", (head, list));
    }
}
