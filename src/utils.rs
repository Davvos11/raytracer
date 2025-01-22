use std::fmt::{format, Debug};
use num_traits::Zero;

pub mod rtweekend;
pub mod scenes;
pub mod parser;

pub fn get_non_zero<T: Zero>(items: &[T]) -> Vec<(usize, &T)> {
    items.iter()
        .enumerate()
        .filter(|(_, x)| !x.is_zero())
        .collect()
}

pub fn debug_buffer<T: Debug + Zero>(items: Option<&[T]>, label: &str) {
    if let Some(items) = items {
        let mut name = format!("{label}:");
        println!("{name:<20} {:?}", &items[0..20]);
        name = format!("{label} (last):");
        println!("{name:<20} {:?}", &items[items.len() - 20..]);
        let items_nonzero = get_non_zero(items);
        name = format!("{label} != 0:");
        println!("{name:<20} {:?}", items_nonzero.iter().take(20).collect::<Vec<_>>());
        println!("{name:<20} {}", items_nonzero.len());
        println!();
    }
}