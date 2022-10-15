struct A(i32);

#[test]
fn test_for() {
    let mut v = vec![1, 2];
    // for mut x in &mut v {
    //     // *x += 1;
    //
    //     for y in &v {
    //         println!("----{}",*x);
    //         // if *x == *y - 1 {
    //         //     *x += *y;
    //         // }
    //     }
    // }

    for x_idx in 0..v.len() {
        for y_idx in 0..v.len() {
            if v[x_idx] == v[y_idx] - 1 {}
        }
    }
}
