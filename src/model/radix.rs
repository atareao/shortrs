
const RADIX: u32 = 36;

pub fn to_d36(mut x: u32) -> String {
    let mut result = vec![];
    loop {
        let m = x % RADIX;
        x = x / RADIX;
        result.push(std::char::from_digit(m, RADIX).unwrap());
        if x == 0 {
            break;
        }
    }
    result.into_iter().rev().collect()
}
pub fn from_d36(x: &str) -> u32{
    let mut value: u32 = 0;
    let n: u32 = x.len().try_into().unwrap();
    for (pos, char) in x.chars().enumerate(){
        let upos: u32 = pos.try_into().unwrap();
        let realpos = n - upos - 1;
        let val: u32 = char.to_digit(RADIX).unwrap().try_into().unwrap();
        value += RADIX.pow(realpos) * val;
    }
    value
}

#[test]
fn sample(){
    println!("{}", from_d36("y"));
    println!("{}", to_d36(1234));
}
