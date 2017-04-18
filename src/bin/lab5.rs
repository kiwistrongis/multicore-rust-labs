use std::thread;

fn main(){
	let mut handles = vec![];
	for _ in 0..5000 {
		handles.push( thread::spawn( do_nothing));}
	for handle in handles {
		let _result = handle.join();}
}

fn do_nothing(){
	let _ = 1;}
