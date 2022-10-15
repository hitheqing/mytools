
#[test]
fn test_linq(){

	let a =vec![1,2,3,4,5];

	// skip_while  的意思是遍历迭代器，如果lamda返回true 则一直添加元素，直到lamda返回false，则结束
	let collect : Vec<i32>= a.iter().take_while(|x1| **x1<3).map(|x2| x2*2).collect();
	// [2,4]
	for x in collect {
		println!("-x = {}",x);
	}

	// skip_while  的意思是遍历迭代器，如果lamda返回true 则一直跳过元素，直到lamda返回false，把剩下的都添加到迭代器里面
	let collect : Vec<i32>= a.iter().skip(1).skip_while(|x1|**x1 < 5).map(|x2| *x2).collect();
	// [10]
	for x in collect {
		println!("--x = {}",x);
	}
}