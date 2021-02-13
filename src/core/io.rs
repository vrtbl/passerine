use crate::common::data::Data;

pub fn println(data: Data) -> Result<Data, String> {
    println!("{}", data);
    return Ok(data);
}

pub fn print(data: Data) -> Result<Data, String> {
    print!("{}", data);
    return Ok(data);
}
