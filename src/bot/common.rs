use std::error::Error;

pub fn print_num_sep3(num: i64) -> Result<String, Box<dyn Error>> {
    let res = num
        .to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()?
        .join(" ");
    Ok(res)
}

