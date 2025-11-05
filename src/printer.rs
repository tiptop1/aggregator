use super::aggregator::Aggregates;


pub fn stdout_print(aggregates: &Aggregates) {
    for category in aggregates.categories() {
        print!("***** {} *****", category);
        match aggregates.fields(category) {
            Some(fields) => fields.iter().for_each(|(k, v)| print!("{}: {}", k, v)),
            _ => ()
        }
    }
}