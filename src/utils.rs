use num_traits::Zero;

pub mod rtweekend;
pub mod scenes;
pub mod parser;

pub fn get_non_zero<T: Zero>(items: &[T], max: Option<usize>) -> Vec<(usize, &T)> {
    items.iter()
        .enumerate()
        .filter(|(_, x)| !x.is_zero())
        .take(max.unwrap_or(usize::MAX))
        .collect()
}