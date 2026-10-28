mod score;
mod store;
fn main() {
    println!("coordinator starting");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_runs() {
        assert_eq!(2 + 2, 4);
    }
}
