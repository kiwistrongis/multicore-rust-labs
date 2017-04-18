extern crate rand;

use std::sync::{ Arc, Barrier, Condvar, Mutex};
use std::sync::atomic::{ AtomicBool, Ordering};
use std::thread;

// constants
const EXCHANGE_DAYS : u32 = 360;
const EXCHANGE_DELAY : u32 = 100;
const BROKERS_N : u32 = 5;

// vars
#[derive( Clone)]
struct Stock {
	cond : Arc<Condvar>,
	mutex : Arc<Mutex<StockInner>>,
}
struct StockInner {
	name : char,
	price : i32,
	amount : u32,
}
impl Stock {
	pub fn new( name : char, price : i32, amount : u32) -> Stock {
		Stock {
			cond: Arc::new( Condvar::new()),
			mutex: Arc::new( Mutex::new(
				StockInner {
					name : name,
					price : price,
					amount : amount}))
		}}
}

struct BrokerArg {
	id: u32,
	buy_price: i32,
	buy_amount: u32,
	sell_price: i32,
	sell_amount: u32,
}
impl BrokerArg {
	pub fn new( id: u32,
		buy_price: i32, buy_amount: u32,
		sell_price: i32, sell_amount: u32) -> BrokerArg {
		BrokerArg {
			id: id,
			buy_price: buy_price,
			buy_amount: buy_amount,
			sell_price: sell_price,
			sell_amount: sell_amount}}
}

// entry-point function
fn main(){
	let stocks = vec![
		Stock::new( 'A', 100,  50),
		Stock::new( 'B', 200, 150),
		Stock::new( 'C', 150,  50),
		Stock::new( 'D', 240, 100),
		Stock::new( 'E', 300, 800),];
	let stocks_n = stocks.len();
	// useless barrier :D
	let barrier = Arc::new( Barrier::new( BROKERS_N));
	// thread-safe boolean access for the win!
	let term = Arc::new( AtomicBool::new( false));


	let mut brokers = vec![];
	for i in 0..BROKERS_N {
		let j = i as usize % stocks_n;
		let x = i as i32 / stocks_n as i32 + 1;
		let stock = stocks[j].clone();
		let mut arg = BrokerArg::new( i as u32, 0, 5, 0, 5);
		{ // temporary scope due to `stock` borrow
			let stock_inner = stock.mutex.lock().unwrap();
			arg.buy_price = stock_inner.price - 10*( 1 + x);
			arg.sell_price = stock_inner.price + 10*( 1 + x);}
		// clone our synonization primitives
		let barrier = barrier.clone();
		let term = term.clone();

		brokers.push( thread::spawn(
			move ||{ thread_broker( stock, arg, term);}
		));}

	// start exchange
	let term = term.clone();
	let exchange = thread::spawn(
		move ||{ thread_exchange( stocks, term);}
	);

	let _ = exchange.join();
	for handle in brokers {
		let _ = handle.join();}
}

// broker thread function
fn thread_broker(
		stock : Stock, arg: BrokerArg, term: Arc<AtomicBool>){
	// vars
	let ( cond, stock_mutex) = ( stock.cond, stock.mutex);
	let mut stock = stock_mutex.lock().unwrap();
	// debug
	println!( "Broker {} watching {}", arg.id, stock.name);

	// start waiting for signals
	// while term signal has not been sent
	loop {
		stock = cond.wait( stock).unwrap();
		// check term signal
		if term.load( Ordering::Relaxed) { break;}
		// if new price is desireable, and there is some left, buy
		if stock.price < arg.buy_price && stock.amount > 0 {
			let change = if stock.amount >= arg.buy_amount {
				arg.buy_amount} else { stock.amount};
			println!( "   > Broker {} bought {:02} {}",
				arg.id, change, stock.name);
			stock.amount -= change;}
		// if new price is lucrative, sell
		else if stock.price > arg.sell_price {
			println!( "   > Broker {} sold {:02} {}",
				arg.id, arg.sell_amount, stock.name);
			stock.amount += arg.sell_amount;}}
	// exit
	println!( "Broker {} terminating", arg.id);
	return;}

// exchange thread function
fn thread_exchange( mut stocks: Vec<Stock>, term : Arc<AtomicBool>){
	// every day
	for day in 0..EXCHANGE_DAYS {
		println!("Day [{:03}]:", day);
		// update the value of each stock and signal waiting threads
		update_stock( &mut stocks[0], one_in( 3), price_var( 10, 3.6, 2.3));
		update_stock( &mut stocks[1], one_in( 7), price_var( 12, 5.0, 2.3));
		update_stock( &mut stocks[2], one_in( 6), price_var(  7, 1.0, 2.1));
		update_stock( &mut stocks[3], one_in( 2), price_var(  8, 5.0, 1.8));
		update_stock( &mut stocks[4], one_in( 4), price_var(  8, 2.0, 1.4));
		#[allow( deprecated)]
		thread::sleep_ms( EXCHANGE_DELAY);}
	// exit
	term.store( true, Ordering::Relaxed);
	for stock in stocks {
		stock.cond.notify_all();}
	println!( "Exchange terminating");
	return;}

// possibly vary stock price, signal brokers on change
fn update_stock( stock : &mut Stock, chance : bool, price_var : i32){
	// if random chance check passes and price hasn't hit minumum
	let mut stock_mut = stock.mutex.lock().unwrap();

	if chance && stock_mut.price > 100 {
		// vary the price randomly
		stock_mut.price += price_var;
		println!("  {}: {:03} @ {:05} {:+04})",
			stock_mut.name, stock_mut. amount, stock_mut.price, price_var);
		// and signal brokers
		stock.cond.notify_all();}
	else {
		println!("  {}: {:03} @ {:05}",
			stock_mut.name, stock_mut. amount, stock_mut.price);}}

// calculate random change 1/n
fn one_in( n: u32) -> bool {
	return 0 == ( rand::random::<u32>() % n);}
// calculate random price variance
fn price_var( a : u32 , b : f32, c : f32) -> i32 {
	let x = rand::random::<u32>() % a;
	let x = x as f32 - b;
	return ( 10f32 * x / c).round() as i32;}