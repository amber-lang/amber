pub mod monomorphize;
pub mod signature;

/// Convert an index to an ordinal number.
pub fn ordinal_number(index: usize) -> String {
    let index = index + 1;
    let mut result = index.to_string();
    let last_digit = index % 10;
    if last_digit == 1 {
        result.push_str("st");
    } else if last_digit == 2 {
        result.push_str("nd");
    } else if last_digit == 3 {
        result.push_str("rd");
    } else {
        result.push_str("th");
    }
    result
}
