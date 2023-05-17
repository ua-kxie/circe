struct Test<'a> {
    v: Vec<usize>,
    r: Option<&'a mut usize>,
}

fn main() {
    let mut t = Test{v: vec![], r: None};
    // t.v.push(5);
    // t.r = t.v.last_mut();  // this line mutably borrows t.v
    let mut a = 3;
    t.r = Some(&mut a);
    // t.r = None;
    t.v.pop();  // this line also mutably borrows t.v
}